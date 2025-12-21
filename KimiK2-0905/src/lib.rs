//! 高精度十进制数库
//!
//! 提供任意精度十进制数的四则运算，支持负号、千分位、小数点等格式。
//!
//! # 示例
//! ```
//! use high_precision_calculations::BigDecimal;
//!
//! let a = BigDecimal::parse("1,234.56").unwrap();
//! let b = BigDecimal::parse("789.12").unwrap();
//! let sum = a.add(&b).unwrap();
//! assert_eq!(sum.to_string(), "2023.68");
//! ```

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Sub};

/// 高精度十进制数
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigDecimal {
    /// 符号：true为正，false为负
    sign: bool,
    /// 数字位（从低位到高位存储）
    digits: Vec<u8>,
    /// 小数位数
    scale: usize,
}

/// 错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum BigDecimalError {
    InvalidFormat(String),
    DivisionByZero,
    InvalidInput(String),
}

impl Display for BigDecimalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BigDecimalError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            BigDecimalError::DivisionByZero => write!(f, "Division by zero"),
            BigDecimalError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl Error for BigDecimalError {}

impl BigDecimal {
    /// 创建零值
    pub fn zero() -> Self {
        BigDecimal {
            sign: true,
            digits: vec![0],
            scale: 0,
        }
    }

    /// 创建一值
    pub fn one() -> Self {
        BigDecimal {
            sign: true,
            digits: vec![1],
            scale: 0,
        }
    }

    /// 判断是否为负数
    pub fn is_negative(&self) -> bool {
        !self.sign && !self.is_zero()
    }

    /// 判断是否为正数
    pub fn is_positive(&self) -> bool {
        self.sign && !self.is_zero()
    }

    /// 判断是否为零
    pub fn is_zero(&self) -> bool {
        self.digits.iter().all(|&d| d == 0)
    }

    /// 获取小数位数
    pub fn scale(&self) -> usize {
        self.scale
    }

    /// 标准化：移除前导零和尾随零
    pub fn normalize(&mut self) {
        // 移除前导零
        while self.digits.len() > 1 && *self.digits.last().unwrap() == 0 {
            self.digits.pop();
        }

        // 如果是零，确保符号为正
        if self.is_zero() {
            self.sign = true;
            self.scale = 0;
        }

        // 移除小数部分尾随零
        while self.scale > 0 {
            // 检查小数部分的最低位（索引0）是否为0
            if !self.digits.is_empty() && self.digits[0] == 0 {
                self.digits.remove(0);
                self.scale -= 1;
            } else {
                break;
            }
        }
    }

    /// 解析字符串为BigDecimal
    pub fn parse(input: &str) -> Result<Self, BigDecimalError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(BigDecimalError::InvalidFormat("Empty input".to_string()));
        }

        let mut sign = true;
        let mut start = 0;

        // 处理符号
        if input.starts_with('-') {
            sign = false;
            start = 1;
        } else if input.starts_with('+') {
            start = 1;
        }

        // 移除千分位逗号
        let mut clean_input = String::new();
        let mut has_decimal = false;
        let mut _decimal_pos = None;

        for ch in input[start..].chars() {
            match ch {
                '0'..='9' => clean_input.push(ch),
                ',' => continue, // 跳过千分位逗号
                '.' => {
                    if has_decimal {
                        return Err(BigDecimalError::InvalidFormat(
                            "Multiple decimal points".to_string(),
                        ));
                    }
                    has_decimal = true;
                    _decimal_pos = Some(clean_input.len());
                    clean_input.push('.');
                }
                _ => {
                    return Err(BigDecimalError::InvalidFormat(format!(
                        "Invalid character: {}",
                        ch
                    )));
                }
            }
        }

        if clean_input.is_empty() {
            return Err(BigDecimalError::InvalidFormat(
                "No digits found".to_string(),
            ));
        }

        // 解析数字和小数位数
        let mut digits = Vec::new();
        let mut scale = 0;
        let mut after_decimal = false;

        for ch in clean_input.chars() {
            if ch == '.' {
                after_decimal = true;
            } else {
                digits.push(ch.to_digit(10).unwrap() as u8);
                if after_decimal {
                    scale += 1;
                }
            }
        }

        // 反转数字（低位在前）
        digits.reverse();

        let mut result = BigDecimal {
            sign,
            digits,
            scale,
        };

        result.normalize();
        Ok(result)
    }

    /// 转换为字符串
    pub fn to_formatted_string(&self) -> String {
        self.to_string_with_grouping(false)
    }

    /// 转换为带千分位的字符串
    pub fn to_string_with_grouping(&self, grouping: bool) -> String {
        if self.is_zero() {
            return "0".to_string();
        }

        let mut result = String::new();
        if !self.sign {
            result.push('-');
        }

        let total_digits = self.digits.len();
        let integer_digits = total_digits.saturating_sub(self.scale);

        // 整数部分
        if integer_digits == 0 {
            result.push('0');
        } else {
            let mut integer_part = Vec::new();
            for i in (self.scale..total_digits).rev() {
                integer_part.push(self.digits[i]);
            }

            if grouping && integer_part.len() > 3 {
                // 添加千分位逗号
                for (i, &digit) in integer_part.iter().enumerate() {
                    if i > 0 && (integer_part.len() - i) % 3 == 0 {
                        result.push(',');
                    }
                    result.push((b'0' + digit) as char);
                }
            } else {
                for &digit in &integer_part {
                    result.push((b'0' + digit) as char);
                }
            }
        }

        // 小数部分
        if self.scale > 0 {
            result.push('.');
            for i in (0..self.scale.min(self.digits.len())).rev() {
                result.push((b'0' + self.digits[i]) as char);
            }
        }

        result
    }

    /// 比较两个BigDecimal的大小
    pub fn compare(&self, other: &Self) -> Ordering {
        // 处理符号不同的情况
        if self.sign != other.sign {
            return if self.sign {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }

        // 处理零的情况
        if self.is_zero() && other.is_zero() {
            return Ordering::Equal;
        }

        // 比较绝对值
        let abs_cmp = self.compare_abs(other);

        if self.sign {
            abs_cmp
        } else {
            abs_cmp.reverse()
        }
    }

    /// 比较绝对值大小
    fn compare_abs(&self, other: &Self) -> Ordering {
        let self_int_digits = self.digits.len().saturating_sub(self.scale);
        let other_int_digits = other.digits.len().saturating_sub(other.scale);

        // 比较整数部分位数
        match self_int_digits.cmp(&other_int_digits) {
            Ordering::Greater => return Ordering::Greater,
            Ordering::Less => return Ordering::Less,
            Ordering::Equal => {}
        }

        // 比较整数部分数字
        for i in (self.scale.max(other.scale)..self.digits.len().max(other.digits.len())).rev() {
            let self_digit = if i < self.digits.len() {
                self.digits[i]
            } else {
                0
            };
            let other_digit = if i < other.digits.len() {
                other.digits[i]
            } else {
                0
            };

            match self_digit.cmp(&other_digit) {
                Ordering::Greater => return Ordering::Greater,
                Ordering::Less => return Ordering::Less,
                Ordering::Equal => {}
            }
        }

        // 比较小数部分
        for i in (0..self.scale.max(other.scale)).rev() {
            let self_digit = if i < self.digits.len() && i < self.scale {
                self.digits[i]
            } else {
                0
            };
            let other_digit = if i < other.digits.len() && i < other.scale {
                other.digits[i]
            } else {
                0
            };

            match self_digit.cmp(&other_digit) {
                Ordering::Greater => return Ordering::Greater,
                Ordering::Less => return Ordering::Less,
                Ordering::Equal => {}
            }
        }

        Ordering::Equal
    }

    /// 加法运算
    pub fn add(&self, other: &Self) -> Result<Self, BigDecimalError> {
        // 处理符号不同的情况
        if self.sign != other.sign {
            if self.sign {
                // self为正，other为负: self - |other|
                return self.subtract_abs(other, true);
            } else {
                // self为负，other为正: other - |self|
                return other.subtract_abs(self, true);
            }
        }

        // 符号相同，直接相加
        self.add_abs(other)
    }

    /// 减法运算
    pub fn sub(&self, other: &Self) -> Result<Self, BigDecimalError> {
        // 转换为加法：self - other = self + (-other)
        let neg_other = BigDecimal {
            sign: !other.sign,
            digits: other.digits.clone(),
            scale: other.scale,
        };
        self.add(&neg_other)
    }

    /// 乘法运算
    pub fn mul(&self, other: &Self) -> Result<Self, BigDecimalError> {
        let mut result = self.multiply_abs(other)?;
        result.sign = self.sign == other.sign;
        result.normalize();
        Ok(result)
    }

    /// 除法运算
    pub fn div(&self, other: &Self) -> Result<Self, BigDecimalError> {
        if other.is_zero() {
            return Err(BigDecimalError::DivisionByZero);
        }

        let mut result = self.divide_abs(other)?;
        result.sign = self.sign == other.sign;
        result.normalize();
        Ok(result)
    }

    /// 绝对值相加
    fn add_abs(&self, other: &Self) -> Result<Self, BigDecimalError> {
        let max_scale = self.scale.max(other.scale);
        let mut result_digits = Vec::new();
        let mut carry = 0;

        // 对齐小数位数
        for i in 0..max_scale {
            let self_digit = if i < self.scale && i < self.digits.len() {
                self.digits[i]
            } else {
                0
            };
            let other_digit = if i < other.scale && i < other.digits.len() {
                other.digits[i]
            } else {
                0
            };

            let sum = self_digit + other_digit + carry;
            result_digits.push(sum % 10);
            carry = sum / 10;
        }

        // 处理整数部分 - 使用安全索引计算
        let self_int_len = self.digits.len().saturating_sub(self.scale);
        let other_int_len = other.digits.len().saturating_sub(other.scale);
        let max_int_digits = self_int_len.max(other_int_len);

        for i in 0..max_int_digits {
            let self_idx = self.scale.saturating_add(i);
            let other_idx = other.scale.saturating_add(i);

            let self_digit = if self_idx < self.digits.len() {
                self.digits[self_idx]
            } else {
                0
            };
            let other_digit = if other_idx < other.digits.len() {
                other.digits[other_idx]
            } else {
                0
            };

            let sum = self_digit + other_digit + carry;
            result_digits.push(sum % 10);
            carry = sum / 10;
        }

        if carry > 0 {
            result_digits.push(carry);
        }

        let mut result = BigDecimal {
            sign: self.sign,
            digits: result_digits,
            scale: max_scale,
        };

        result.normalize();
        Ok(result)
    }

    /// 绝对值相减
    fn subtract_abs(&self, other: &Self, swap_on_less: bool) -> Result<Self, BigDecimalError> {
        let cmp = self.compare_abs(other);

        let (larger, smaller, should_negate) = match cmp {
            Ordering::Greater => (self, other, false),
            Ordering::Less => {
                if swap_on_less {
                    (other, self, true)
                } else {
                    return Ok(BigDecimal::zero());
                }
            }
            Ordering::Equal => return Ok(BigDecimal::zero()),
        };

        let max_scale = larger.scale.max(smaller.scale);
        let mut result_digits = Vec::new();
        let mut borrow = 0;

        // 对齐小数位数相减
        for i in 0..max_scale {
            let larger_digit = if i < larger.scale && i < larger.digits.len() {
                larger.digits[i]
            } else {
                0
            };
            let smaller_digit = if i < smaller.scale && i < smaller.digits.len() {
                smaller.digits[i]
            } else {
                0
            };

            let mut diff = larger_digit as i16 - smaller_digit as i16 - borrow as i16;
            if diff < 0 {
                diff += 10;
                borrow = 1;
            } else {
                borrow = 0;
            }
            result_digits.push(diff as u8);
        }

        // 整数部分相减 - 使用更安全的索引计算
        let larger_int_len = larger.digits.len().saturating_sub(larger.scale);
        let smaller_int_len = smaller.digits.len().saturating_sub(smaller.scale);
        let max_int_digits = larger_int_len.max(smaller_int_len);

        for i in 0..max_int_digits {
            let larger_idx = larger.scale.saturating_add(i);
            let smaller_idx = smaller.scale.saturating_add(i);

            let larger_digit = if larger_idx < larger.digits.len() {
                larger.digits[larger_idx]
            } else {
                0
            };
            let smaller_digit = if smaller_idx < smaller.digits.len() {
                smaller.digits[smaller_idx]
            } else {
                0
            };

            let mut diff = larger_digit as i16 - smaller_digit as i16 - borrow as i16;
            if diff < 0 {
                diff += 10;
                borrow = 1;
            } else {
                borrow = 0;
            }
            result_digits.push(diff as u8);
        }

        let sign = if should_negate {
            !larger.sign
        } else {
            larger.sign
        };
        let mut result = BigDecimal {
            sign,
            digits: result_digits,
            scale: max_scale,
        };

        result.normalize();
        Ok(result)
    }

    /// 绝对值相乘
    fn multiply_abs(&self, other: &Self) -> Result<Self, BigDecimalError> {
        let mut result_digits = vec![0; self.digits.len() + other.digits.len()];

        for i in 0..self.digits.len() {
            let mut carry = 0;
            for j in 0..other.digits.len() {
                let product = self.digits[i] as u16 * other.digits[j] as u16
                    + result_digits[i + j] as u16
                    + carry;
                result_digits[i + j] = (product % 10) as u8;
                carry = product / 10;
            }
            if carry > 0 {
                result_digits[i + other.digits.len()] += carry as u8;
            }
        }

        let mut result = BigDecimal {
            sign: true,
            digits: result_digits,
            scale: self.scale + other.scale,
        };

        result.normalize();
        Ok(result)
    }

    /// 绝对值相除（使用长除法）
    fn divide_abs(&self, other: &Self) -> Result<Self, BigDecimalError> {
        const PRECISION: usize = 20; // 除法精度

        // 如果除数为零，返回错误
        if other.is_zero() {
            return Err(BigDecimalError::DivisionByZero);
        }

        // 简单的除法实现
        let mut remainder = self.clone();

        // 计算整数部分
        let mut integer_quotient = 0;
        while remainder.compare(other) != Ordering::Less {
            remainder = remainder.sub(other)?;
            integer_quotient += 1;
            if integer_quotient > 1000 {
                // 防止无限循环
                break;
            }
        }

        let mut result = BigDecimal::parse(&integer_quotient.to_string()).unwrap();

        // 计算小数部分
        for _i in 0..PRECISION {
            if remainder.is_zero() {
                break;
            }

            // 乘以10
            remainder = remainder.mul(&BigDecimal::parse("10").unwrap())?;

            let mut digit = 0;
            while remainder.compare(other) != Ordering::Less {
                remainder = remainder.sub(other)?;
                digit += 1;
                if digit >= 10 {
                    break;
                }
            }

            // 添加小数位
            let digit_decimal = BigDecimal::parse(&format!("{}.{}", 0, digit)).unwrap();
            result = result.add(&digit_decimal)?;
        }

        result.normalize();
        Ok(result)
    }
}

// 实现运算符重载
impl Add for &BigDecimal {
    type Output = Result<BigDecimal, BigDecimalError>;

    fn add(self, other: Self) -> Self::Output {
        self.add(other)
    }
}

impl Sub for &BigDecimal {
    type Output = Result<BigDecimal, BigDecimalError>;

    fn sub(self, other: Self) -> Self::Output {
        self.sub(other)
    }
}

impl Mul for &BigDecimal {
    type Output = Result<BigDecimal, BigDecimalError>;

    fn mul(self, other: Self) -> Self::Output {
        self.mul(other)
    }
}

impl Div for &BigDecimal {
    type Output = Result<BigDecimal, BigDecimalError>;

    fn div(self, other: Self) -> Self::Output {
        self.div(other)
    }
}

impl fmt::Display for BigDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_formatted_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let num = BigDecimal::parse("123.45").unwrap();
        assert_eq!(num.to_string(), "123.45");
        assert_eq!(num.scale(), 2);
    }

    #[test]
    fn test_parse_negative() {
        let num = BigDecimal::parse("-123.45").unwrap();
        assert!(num.is_negative());
        assert_eq!(num.to_string(), "-123.45");
    }

    #[test]
    fn test_parse_with_commas() {
        let num = BigDecimal::parse("1,234,567.89").unwrap();
        assert_eq!(num.to_string(), "1234567.89");
    }

    #[test]
    fn test_addition() {
        let a = BigDecimal::parse("123.45").unwrap();
        let b = BigDecimal::parse("67.89").unwrap();
        let sum = a.add(&b).unwrap();
        assert_eq!(sum.to_string(), "191.34");
    }

    #[test]
    fn test_subtraction() {
        let a = BigDecimal::parse("123.45").unwrap();
        let b = BigDecimal::parse("67.89").unwrap();
        let diff = a.sub(&b).unwrap();
        assert_eq!(diff.to_string(), "55.56");
    }

    #[test]
    fn test_multiplication() {
        let a = BigDecimal::parse("12.34").unwrap();
        let b = BigDecimal::parse("5.67").unwrap();
        let product = a.mul(&b).unwrap();
        assert_eq!(product.to_string(), "69.9678");
    }

    #[test]
    fn test_division() {
        let a = BigDecimal::parse("100.0").unwrap();
        let b = BigDecimal::parse("4.0").unwrap();
        let quotient = a.div(&b).unwrap();
        assert_eq!(quotient.to_string(), "25");
    }

    #[test]
    fn test_zero() {
        let zero = BigDecimal::zero();
        assert!(zero.is_zero());
        // 零不被认为是正数
        assert_eq!(zero.to_string(), "0");
    }

    #[test]
    fn test_normalize() {
        let mut num = BigDecimal::parse("00123.4500").unwrap();
        num.normalize();
        assert_eq!(num.to_string(), "123.45");
    }
}
