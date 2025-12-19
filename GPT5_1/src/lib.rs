//! 高精度十进制四则运算核心库。
//!
//! 设计目标：
//! - 复刻旧 C++ 链表实现的**对外行为**（解析规则、舍入/截断、ERROR 条件等）；
//! - 同时用 Rust 类型系统消除：裸指针、隐式副作用、魔法数字散落、重复代码等代码异味；
//! - 提供可复用的 `BigDecimal` 类型与清晰 API，而不是过程化的全局函数。

use std::cmp::Ordering;
use std::fmt;

/// 除法内部固定扩展的小数位数（对应旧代码中 20 的魔法数字）。
pub const DIV_FRACTIONAL_DIGITS: usize = 20;
/// 乘除输出阶段最多打印的小数位数字，用于复刻旧 `print_2` 的“10 位 + 古怪四舍五入”逻辑。
pub const LEGACY_ROUND_DIGITS: usize = 10;
/// 旧 `toosmall` 对除数做“过小”判断时，最多检查的前缀位数（对应魔法数字 7）。
pub const TOO_SMALL_PREFIX_DIGITS: usize = 7;

/// 十进制数的符号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    Positive,
    Negative,
}

impl Sign {
    fn from_is_negative(is_negative: bool) -> Self {
        if is_negative {
            Sign::Negative
        } else {
            Sign::Positive
        }
    }

    fn is_negative(self) -> bool {
        matches!(self, Sign::Negative)
    }
}

/// 高精度小数错误类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Empty,
    InvalidChar(char),
    MultipleDecimalPoints,
    NoDigits,
    /// 对应旧实现中的 `ERROR` 输出（除以 0 或除数过小）。
    DivisionByZeroOrTooSmall,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Empty => write!(f, "input is empty"),
            Error::InvalidChar(ch) => write!(f, "invalid character: {}", ch),
            Error::MultipleDecimalPoints => write!(f, "multiple decimal points"),
            Error::NoDigits => write!(f, "input contains no digits"),
            Error::DivisionByZeroOrTooSmall => write!(f, "division by zero or too small"),
        }
    }
}

impl std::error::Error for Error {}

/// 任意精度十进制数。
///
/// 不变量：
/// - `digits` 为 **大端** 存储（最高位在前），基数 10；
/// - 每一位都在 0..=9 之间，不携带符号信息；
/// - 除非值为 0，否则 `digits` 不含前导 0；
/// - `scale` 表示小数位数，可以为负数（主要由除法产生的中间形式带来，最终经 `normalize` 恢复为非负）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigDecimal {
    pub sign: Sign,
    pub digits: Vec<u8>,
    pub scale: i32,
}

impl BigDecimal {
    /// 从字符串解析高精度十进制数。
    ///
    /// 支持：
    /// - 可选前导 `+` / `-`；
    /// - 千分位分隔符 `,`（解析时直接忽略，不做位置合法性校验）；
    /// - 一个小数点 `.`；
    pub fn parse(input: &str) -> Result<Self, Error> {
        let s = input.trim();
        if s.is_empty() {
            return Err(Error::Empty);
        }

        let mut sign = Sign::Positive;
        let mut digits = Vec::new();
        let mut scale: i32 = 0;
        let mut seen_digit = false;
        let mut seen_dot = false;

        for (idx, ch) in s.chars().enumerate() {
            if idx == 0 && (ch == '-' || ch == '+') {
                if ch == '-' {
                    sign = Sign::Negative;
                }
                continue;
            }
            match ch {
                ',' => continue,
                '.' => {
                    if seen_dot {
                        return Err(Error::MultipleDecimalPoints);
                    }
                    seen_dot = true;
                }
                '0'..='9' => {
                    seen_digit = true;
                    digits.push(ch as u8 - b'0');
                    if seen_dot {
                        scale += 1;
                    }
                }
                _ => return Err(Error::InvalidChar(ch)),
            }
        }

        if !seen_digit {
            return Err(Error::NoDigits);
        }

        let mut result = BigDecimal { sign, digits, scale };
        result.normalize();
        Ok(result)
    }

    /// 归一化内部表示：
    /// - 去掉整数部分前导 0；
    /// - 如果数值为 0，则强制为 `+0` 且 `scale = 0`。
    pub fn normalize(&mut self) {
        trim_leading_zeros(&mut self.digits);
        if self.is_zero() {
            self.sign = Sign::Positive;
            self.scale = 0;
        }
    }

    /// 判断数值绝对值是否为 0。
    pub fn is_zero(&self) -> bool {
        self.digits.len() == 1 && self.digits[0] == 0
    }

    /// 旧实现中 `toosmall` 的“除数过小”判断。
    ///
    /// 逻辑：把小数展开到至少 `scale + 1` 位，然后检查前 `TOO_SMALL_PREFIX_DIGITS`
    /// 位是否全部为 0；如果是，则认为“过小”（返回 `true`）。
    pub fn is_too_small_divisor(&self) -> bool {
        if self.is_zero() {
            return true;
        }

        let mut digits = self.digits.clone();
        let scale = if self.scale < 0 { 0 } else { self.scale as usize };
        pad_leading_zeros(&mut digits, scale);

        let check_len = TOO_SMALL_PREFIX_DIGITS.min(digits.len());
        digits.iter().take(check_len).all(|&d| d == 0)
    }

    /// 加法，时间复杂度 \(O(n)\)。
    pub fn add(&self, other: &Self) -> Self {
        let left = self.normalized_for_ops();
        let right = other.normalized_for_ops();
        let (a, b, scale) = align_scales(&left, &right);

        let (sign, digits) = match (left.sign, right.sign) {
            (Sign::Positive, Sign::Positive) => (Sign::Positive, add_digits(&a, &b)),
            (Sign::Negative, Sign::Negative) => (Sign::Negative, add_digits(&a, &b)),
            _ => match cmp_digits(&a, &b) {
                Ordering::Equal => (Sign::Positive, vec![0]),
                Ordering::Greater => (left.sign, sub_digits(&a, &b)),
                Ordering::Less => (right.sign, sub_digits(&b, &a)),
            },
        };

        let mut result = BigDecimal { sign, digits, scale: scale as i32 };
        result.normalize();
        result
    }

    /// 减法，时间复杂度 \(O(n)\)。
    pub fn sub(&self, other: &Self) -> Self {
        let mut rhs = other.clone();
        rhs.sign = Sign::from_is_negative(!rhs.sign.is_negative());
        self.add(&rhs)
    }

    /// 乘法，时间复杂度 \(O(n \times m)\)。
    pub fn mul(&self, other: &Self) -> Self {
        let left = self.normalized_for_ops();
        let right = other.normalized_for_ops();
        let digits = mul_digits(&left.digits, &right.digits);
        let scale = left.scale + right.scale;
        let sign = if digits.len() == 1 && digits[0] == 0 {
            Sign::Positive
        } else {
            Sign::from_is_negative(left.sign != right.sign)
        };

        let mut result = BigDecimal { sign, digits, scale };
        result.normalize();
        result
    }

    /// 除法，复刻旧实现的精度与 `ERROR` 规则。
    ///
    /// 算法：
    /// - 先按整数长除法，对被除数补上 `DIV_FRACTIONAL_DIGITS` 个 0；
    /// - 最终结果小数位数为 `DIV_FRACTIONAL_DIGITS + self.scale - other.scale`；
    /// - 除数过小或为 0 时返回 `Error::DivisionByZeroOrTooSmall`。
    pub fn div(&self, other: &Self) -> Result<Self, Error> {
        let left = self.normalized_for_ops();
        let right = other.normalized_for_ops();

        if right.is_too_small_divisor() {
            return Err(Error::DivisionByZeroOrTooSmall);
        }

        if left.is_zero() {
            return Ok(BigDecimal {
                sign: Sign::Positive,
                digits: vec![0],
                scale: 0,
            });
        }

        let mut numerator = left.digits.clone();
        numerator.extend(std::iter::repeat(0).take(DIV_FRACTIONAL_DIGITS));
        let quotient = div_digits(&numerator, &right.digits);
        let scale = DIV_FRACTIONAL_DIGITS as i32 + left.scale - right.scale;
        let sign = Sign::from_is_negative(left.sign != right.sign);

        let mut result = BigDecimal { sign, digits: quotient, scale };
        result.normalize();
        Ok(result)
    }

    /// 规范化字符串格式（不做旧 C++ 的怪异行为），可选千分位分隔。
    pub fn to_string_with_grouping(&self, grouping: bool) -> String {
        format_canonical(self, grouping)
    }

    /// 旧行为格式化模式。
    pub fn to_string_legacy(&self, mode: LegacyFormat) -> String {
        format_legacy(self, mode)
    }

    /// 将 `scale < 0` 的中间结果展开为 `scale >= 0` 的形式，便于统一算术运算。
    fn normalized_for_ops(&self) -> BigDecimal {
        if self.scale >= 0 {
            return self.clone();
        }

        let mut digits = self.digits.clone();
        digits.extend(std::iter::repeat(0).take((-self.scale) as usize));
        let mut normalized = BigDecimal {
            sign: self.sign,
            digits,
            scale: 0,
        };
        normalized.normalize();
        normalized
    }
}

/// 旧行为格式化模式：加减 / 乘除。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegacyFormat {
    AddSub,
    MulDiv,
}

fn align_scales(a: &BigDecimal, b: &BigDecimal) -> (Vec<u8>, Vec<u8>, usize) {
    let a_scale = a.scale.max(0) as usize;
    let b_scale = b.scale.max(0) as usize;
    let scale = a_scale.max(b_scale);

    let mut left = a.digits.clone();
    let mut right = b.digits.clone();

    if a_scale < scale {
        left.extend(std::iter::repeat(0).take(scale - a_scale));
    }
    if b_scale < scale {
        right.extend(std::iter::repeat(0).take(scale - b_scale));
    }

    (left, right, scale)
}

fn cmp_digits(a: &[u8], b: &[u8]) -> Ordering {
    if a.len() != b.len() {
        return a.len().cmp(&b.len());
    }
    for (&da, &db) in a.iter().zip(b.iter()) {
        if da != db {
            return da.cmp(&db);
        }
    }
    Ordering::Equal
}

fn add_digits(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(a.len().max(b.len()) + 1);
    let mut carry = 0u8;

    let mut ia = a.len() as i32 - 1;
    let mut ib = b.len() as i32 - 1;
    while ia >= 0 || ib >= 0 || carry != 0 {
        let da = if ia >= 0 { a[ia as usize] } else { 0 };
        let db = if ib >= 0 { b[ib as usize] } else { 0 };
        let sum = da as u16 + db as u16 + carry as u16;
        result.push((sum % 10) as u8);
        carry = (sum / 10) as u8;
        ia -= 1;
        ib -= 1;
    }

    result.reverse();
    trim_leading_zeros(&mut result);
    result
}

fn sub_digits(a: &[u8], b: &[u8]) -> Vec<u8> {
    // 假定 a >= b。
    let mut result = Vec::with_capacity(a.len());
    let mut borrow = 0i8;

    let mut ia = a.len() as i32 - 1;
    let mut ib = b.len() as i32 - 1;
    while ia >= 0 {
        let mut da = a[ia as usize] as i8 - borrow;
        let db = if ib >= 0 { b[ib as usize] as i8 } else { 0 };
        if da < db {
            da += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        let diff = da - db;
        result.push(diff as u8);
        ia -= 1;
        ib -= 1;
    }

    result.reverse();
    trim_leading_zeros(&mut result);
    result
}

fn mul_digits(a: &[u8], b: &[u8]) -> Vec<u8> {
    if (a.len() == 1 && a[0] == 0) || (b.len() == 1 && b[0] == 0) {
        return vec![0];
    }

    let mut acc = vec![0u32; a.len() + b.len()];
    for (i, &da) in a.iter().enumerate() {
        for (j, &db) in b.iter().enumerate() {
            acc[i + j + 1] += (da as u32) * (db as u32);
        }
    }

    for k in (1..acc.len()).rev() {
        let carry = acc[k] / 10;
        acc[k] %= 10;
        acc[k - 1] += carry;
    }

    let mut result: Vec<u8> = acc.into_iter().map(|v| v as u8).collect();
    trim_leading_zeros(&mut result);
    result
}

fn mul_by_digit(a: &[u8], digit: u8) -> Vec<u8> {
    if digit == 0 {
        return vec![0];
    }
    let mut result = Vec::with_capacity(a.len() + 1);
    let mut carry = 0u8;
    for &da in a.iter().rev() {
        let prod = da as u16 * digit as u16 + carry as u16;
        result.push((prod % 10) as u8);
        carry = (prod / 10) as u8;
    }
    if carry != 0 {
        result.push(carry);
    }
    result.reverse();
    trim_leading_zeros(&mut result);
    result
}

fn div_digits(numerator: &[u8], denominator: &[u8]) -> Vec<u8> {
    if denominator.len() == 1 && denominator[0] == 0 {
        return vec![0];
    }

    let mut quotient = Vec::with_capacity(numerator.len());
    let mut remainder: Vec<u8> = Vec::new();

    for &digit in numerator {
        if remainder.len() == 1 && remainder[0] == 0 {
            remainder.clear();
        }
        remainder.push(digit);
        trim_leading_zeros(&mut remainder);

        let mut q = 0u8;
        if cmp_digits(&remainder, denominator) != Ordering::Less {
            for cand in (1u8..=9u8).rev() {
                let prod = mul_by_digit(denominator, cand);
                if cmp_digits(&prod, &remainder) != Ordering::Greater {
                    q = cand;
                    remainder = sub_digits(&remainder, &prod);
                    break;
                }
            }
        }
        quotient.push(q);
    }

    trim_leading_zeros(&mut quotient);
    quotient
}

fn trim_leading_zeros(digits: &mut Vec<u8>) {
    let mut idx = 0;
    while idx + 1 < digits.len() && digits[idx] == 0 {
        idx += 1;
    }
    if idx > 0 {
        digits.drain(0..idx);
    }
}

fn trim_fractional_zeros(digits: &mut Vec<u8>, scale: &mut usize) {
    while *scale > 0 {
        if digits.last().copied() == Some(0) {
            digits.pop();
            *scale -= 1;
        } else {
            break;
        }
    }
    if digits.is_empty() {
        digits.push(0);
        *scale = 0;
    }
}

fn pad_leading_zeros(digits: &mut Vec<u8>, scale: usize) {
    let min_len = scale + 1;
    if digits.len() < min_len {
        let mut padded = vec![0u8; min_len - digits.len()];
        padded.extend_from_slice(digits);
        *digits = padded;
    }
}

fn trim_leading_zeros_with_budget(digits: &mut Vec<u8>, budget: usize) {
    let mut remaining = budget;
    let mut idx = 0;
    while remaining > 0 && idx + 1 < digits.len() && digits[idx] == 0 {
        idx += 1;
        remaining -= 1;
    }
    if idx > 0 {
        digits.drain(0..idx);
    }
}

fn format_canonical(value: &BigDecimal, grouping: bool) -> String {
    let mut digits = value.digits.clone();
    let mut scale = value.scale;

    if scale < 0 {
        digits.extend(std::iter::repeat(0).take((-scale) as usize));
        scale = 0;
    }

    let scale = scale as usize;
    pad_leading_zeros(&mut digits, scale);

    let int_len = digits.len() - scale;
    let mut out = String::new();

    if value.sign == Sign::Negative && !value.is_zero() {
        out.push('-');
    }

    for i in 0..int_len {
        if grouping && i > 0 && (int_len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(char::from(b'0' + digits[i]));
    }

    if scale > 0 {
        out.push('.');
        for d in &digits[int_len..] {
            out.push(char::from(b'0' + *d));
        }
    }

    out
}

fn format_legacy(value: &BigDecimal, mode: LegacyFormat) -> String {
    let mut digits = value.digits.clone();
    let mut scale = if value.scale < 0 { 0 } else { value.scale as usize };

    trim_fractional_zeros(&mut digits, &mut scale);
    pad_leading_zeros(&mut digits, scale);

    let integer_len = digits.len() as i32 - scale as i32;
    let budget = match mode {
        LegacyFormat::AddSub => integer_len,
        LegacyFormat::MulDiv => integer_len - 1,
    };
    if budget > 0 {
        trim_leading_zeros_with_budget(&mut digits, budget as usize);
    }

    if digits.is_empty() {
        digits.push(0);
        scale = 0;
    }

    let all_zero = digits.iter().all(|&d| d == 0);
    let sign = if all_zero { Sign::Positive } else { value.sign };

    let signed_digits: Vec<i8> = digits
        .iter()
        .map(|&d| if sign.is_negative() { -(d as i8) } else { d as i8 })
        .collect();

    match mode {
        LegacyFormat::AddSub => format_legacy_add_sub(&signed_digits, scale, true),
        LegacyFormat::MulDiv => format_legacy_mul_div(&signed_digits, scale, true),
    }
}

fn format_legacy_add_sub(digits: &[i8], scale: usize, grouping: bool) -> String {
    if digits.is_empty() {
        return "0".to_string();
    }

    let n = digits.len() as i32;
    let x = scale as i32;
    let integer_len = n - x;
    let mut out = String::new();

    let first = digits[0];
    if first < 0 {
        out.push('-');
        out.push(char::from(b'0' + (-first) as u8));
    } else {
        out.push(char::from(b'0' + first as u8));
    }

    let mut i = 1i32;
    let mut j = integer_len / 3;
    if i == integer_len && x != 0 {
        out.push('.');
    }

    for idx in 1..digits.len() {
        let d = digits[idx].abs() as u8;
        if grouping {
            if j <= 0 || i % 3 != integer_len % 3 || integer_len <= 3 {
                out.push(char::from(b'0' + d));
            } else if i < integer_len {
                out.push(',');
                out.push(char::from(b'0' + d));
                j -= 1;
            } else {
                out.push(char::from(b'0' + d));
            }
        } else {
            out.push(char::from(b'0' + d));
        }

        i += 1;
        if i == integer_len && x != 0 {
            out.push('.');
        }
    }

    out
}

fn format_legacy_mul_div(digits: &[i8], scale: usize, grouping: bool) -> String {
    if digits.is_empty() {
        return "0".to_string();
    }

    let n = digits.len() as i32;
    let x = scale as i32;
    let integer_len = n - x;
    let mut out = String::new();

    let first = digits[0];
    if first < 0 {
        out.push('-');
        out.push(char::from(b'0' + (-first) as u8));
    } else {
        out.push(char::from(b'0' + first as u8));
    }

    let mut i = 1i32;
    let mut j = integer_len / 3;
    let mut c = i32::MAX;

    if i == integer_len && x != 0 {
        c = (LEGACY_ROUND_DIGITS - 1) as i32;
        out.push('.');
    }

    let mut idx = 1usize;
    while idx < digits.len() && c > 0 {
        c -= 1;
        let d = digits[idx].abs() as u8;
        if grouping {
            if j <= 0 || i % 3 != integer_len % 3 || integer_len <= 3 {
                out.push(char::from(b'0' + d));
            } else if i < integer_len {
                out.push(',');
                out.push(char::from(b'0' + d));
                j -= 1;
            } else {
                out.push(char::from(b'0' + d));
            }
        } else {
            out.push(char::from(b'0' + d));
        }

        i += 1;
        idx += 1;
        if i == integer_len && x != 0 {
            out.push('.');
            c = (LEGACY_ROUND_DIGITS - 1) as i32;
        }
    }

    if c == 0 {
        if idx >= digits.len() {
            return out;
        }
        let cur = digits[idx].abs() as u8;
        if idx + 1 >= digits.len() {
            out.push_str(&cur.to_string());
        } else {
            let next = digits[idx + 1].abs() as u8;
            if next < 5 {
                out.push_str(&cur.to_string());
            } else {
                out.push_str(&(cur + 1).to_string());
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy_add(a: &str, b: &str) -> String {
        let left = BigDecimal::parse(a).unwrap();
        let right = BigDecimal::parse(b).unwrap();
        left.add(&right).to_string_legacy(LegacyFormat::AddSub)
    }

    fn legacy_sub(a: &str, b: &str) -> String {
        let left = BigDecimal::parse(a).unwrap();
        let right = BigDecimal::parse(b).unwrap();
        left.sub(&right).to_string_legacy(LegacyFormat::AddSub)
    }

    fn legacy_mul(a: &str, b: &str) -> String {
        let left = BigDecimal::parse(a).unwrap();
        let right = BigDecimal::parse(b).unwrap();
        left.mul(&right).to_string_legacy(LegacyFormat::MulDiv)
    }

    fn legacy_div(a: &str, b: &str) -> Result<String, String> {
        let left = BigDecimal::parse(a).unwrap();
        let right = BigDecimal::parse(b).unwrap();
        match left.div(&right) {
            Ok(v) => Ok(v.to_string_legacy(LegacyFormat::MulDiv)),
            Err(_) => Err("ERROR".to_string()),
        }
    }

    #[test]
    fn legacy_add_sub_examples() {
        assert_eq!(legacy_add("1,234.5", "6.70"), "1,241.2");
        assert_eq!(legacy_add("0.1", "0.2"), "3");
        assert_eq!(legacy_sub("0.5", "0.2"), "3");
        assert_eq!(legacy_add("-0.1", "-0.2"), "-3");
    }

    #[test]
    fn legacy_mul_examples() {
        assert_eq!(legacy_mul("1.2", "3.4"), "4.08");
        assert_eq!(legacy_mul("1.234567890195", "1"), "1.2345678902");
        assert_eq!(legacy_mul("0.99999999995", "1"), "0.99999999910");
        assert_eq!(legacy_mul("0.1", "-0.2"), "0.02");
    }

    #[test]
    fn legacy_div_examples() {
        assert_eq!(legacy_div("10", "3").unwrap(), "3.3333333333");
        assert_eq!(legacy_div("1", "8").unwrap(), "0.125");
        assert!(legacy_div("1", "0.0000001").is_err());
    }

    #[test]
    fn canonical_formatting() {
        let v = BigDecimal::parse("0.0100").unwrap();
        assert_eq!(v.to_string_with_grouping(false), "0.0100");
    }
}


