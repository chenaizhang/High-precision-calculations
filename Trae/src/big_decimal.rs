use std::cmp::{Ordering, max};
use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Neg};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Positive,
    Negative,
}

impl Sign {
    pub fn negate(self) -> Self {
        match self {
            Sign::Positive => Sign::Negative,
            Sign::Negative => Sign::Positive,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigDecimal {
    pub sign: Sign,
    pub digits: Vec<u8>, // Little-endian: index 0 is the least significant digit (10^0 relative to scale)
    pub scale: i32,      // Number of digits after the decimal point. Changed to i32 to support division logic better.
}

#[derive(Debug, Clone)]
pub enum BigIntError {
    ParseError(String),
    DivisionByZero,
    TooSmallDivisor,
}

impl fmt::Display for BigIntError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BigIntError::ParseError(s) => write!(f, "Parse Error: {}", s),
            BigIntError::DivisionByZero => write!(f, "Division by Zero"),
            BigIntError::TooSmallDivisor => write!(f, "ERROR"), // Match C++ output for too small divisor
        }
    }
}

impl std::error::Error for BigIntError {}

impl BigDecimal {
    pub fn new(sign: Sign, digits: Vec<u8>, scale: i32) -> Self {
        let mut decimal = BigDecimal { sign, digits, scale };
        decimal.normalize();
        decimal
    }

    #[allow(dead_code)]
    pub fn zero() -> Self {
        BigDecimal {
            sign: Sign::Positive,
            digits: vec![0],
            scale: 0,
        }
    }

    /// Removes trailing zeros in the fractional part and leading zeros in the integer part.
    pub fn normalize(&mut self) {
        // 1. Remove trailing zeros (which are at the beginning of the vector in Little Endian)
        // ONLY if they are part of the scale.
        loop {
            if self.scale <= 0 {
                break;
            }
            if let Some(&0) = self.digits.first() {
                self.digits.remove(0);
                self.scale -= 1;
            } else if self.digits.is_empty() {
                // If digits is empty, it means 0. We can reduce scale.
                self.scale -= 1;
            } else {
                break;
            }
        }

        // 2. Remove leading zeros (which are at the end of the vector)
        while self.digits.len() > 1 && self.digits.last() == Some(&0) {
            self.digits.pop();
        }
        
        // Ensure not empty
        if self.digits.is_empty() {
            self.digits.push(0);
        }
        
        // Handle -0
        if self.digits.len() == 1 && self.digits[0] == 0 {
            self.sign = Sign::Positive;
        }
    }
    
    /// Parses a string into a BigDecimal.
    pub fn parse(input: &str) -> Result<Self, BigIntError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(BigIntError::ParseError("Empty string".to_string()));
        }

        let mut sign = Sign::Positive;
        let mut decimal_point_index = None;
        let mut has_digit = false;

        let chars: Vec<char> = input.chars().collect();
        let start_idx = if chars[0] == '-' {
            sign = Sign::Negative;
            1
        } else if chars[0] == '+' {
            1
        } else {
            0
        };

        let mut filtered_chars = Vec::new();
        for &c in &chars[start_idx..] {
            if c == ',' {
                continue;
            }
            if c == '.' {
                if decimal_point_index.is_some() {
                    return Err(BigIntError::ParseError("Multiple decimal points".to_string()));
                }
                decimal_point_index = Some(filtered_chars.len());
                continue;
            }
            if let Some(d) = c.to_digit(10) {
                filtered_chars.push(d as u8);
                has_digit = true;
            } else {
                return Err(BigIntError::ParseError(format!("Invalid character: {}", c)));
            }
        }

        if !has_digit {
             return Err(BigIntError::ParseError("No digits found".to_string()));
        }

        let scale = if let Some(idx) = decimal_point_index {
            (filtered_chars.len() - idx) as i32
        } else {
            0
        };

        let digits: Vec<u8> = filtered_chars.into_iter().rev().collect();
        
        Ok(BigDecimal::new(sign, digits, scale))
    }

    pub fn to_string_with_grouping(&self, use_grouping: bool, max_decimal_places: Option<usize>) -> String {
        let mut s = String::new();
        
        let digits_len = self.digits.len() as i32;
        let scale = self.scale;
        
        // Integer part length
        let int_len = if digits_len > scale {
            digits_len - scale
        } else {
            0
        };

        // Handle sign
        // Legacy behavior: For Mul/Div (max_decimal_places is Some), if integer part is 0, swallow negative sign.
        let skip_sign = max_decimal_places.is_some() && int_len == 0;
        if self.sign == Sign::Negative && !skip_sign {
            s.push('-');
        }

        if int_len == 0 {
            // Legacy behavior: For Add/Sub (max_decimal_places is None), if integer part is 0, swallow the leading '0'.
            if max_decimal_places.is_some() {
                s.push('0');
            }
        } else {
            // Print integer digits
            for i in (0..int_len).rev() {
                let digit_idx = (scale + i) as usize;
                s.push((self.digits[digit_idx] + b'0') as char);
                
                if use_grouping && i > 0 && i % 3 == 0 {
                    s.push(',');
                }
            }
        }

        if scale > 0 {
            // Legacy behavior: For Add/Sub, if integer part is 0, swallow the decimal point.
            let skip_dot = max_decimal_places.is_none() && int_len == 0;
            if !skip_dot {
                s.push('.');
            }
            
            // Logic for limited precision (Legacy C++ print_2 behavior)
            // It prints up to `max` digits.
            // C++: c=9. Loop runs while c>0. So prints 9 digits.
            // Then checks rounding.
            // So visible digits = 9 + 1 (rounding digit) = 10 digits max.
            // However, the "rounding digit" logic is specific.
            
            if let Some(limit) = max_decimal_places {
                // limit is 10 for print_2.
                // We print limit-1 digits normally.
                // Then special handling for limit-th digit.
                
                let _visible_digits = if scale > limit as i32 { limit as i32 } else { scale };
                
                // Leading zeros after decimal point logic needs to be integrated.
                // If scale > digits_len, we have `scale - digits_len` zeros.
                
                // Let's iterate decimal places from 1 to scale.
                let mut current_scale_pos = scale;
                let mut printed_count = 0;
                
                // We need to fetch digit at `current_scale_pos`.
                // Digit index = current_scale_pos - 1.
                // But we must handle leading zeros if `current_scale_pos > digits_len`.
                
                // If limit is triggered, we stop early.
                let truncation_threshold = limit; // e.g. 10
                
                while current_scale_pos > 0 {
                    if printed_count == truncation_threshold - 1 && current_scale_pos > 0 {
                         // This is the 10th digit. Special rounding logic.
                         // We need value of 10th digit and 11th digit.
                         
                         let get_digit = |pos: i32| -> u8 {
                             if pos > digits_len {
                                 0
                             } else if pos > 0 {
                                 self.digits[(pos - 1) as usize]
                             } else {
                                 0
                             }
                         };
                         
                         let digit_10 = get_digit(current_scale_pos);
                         let digit_11 = get_digit(current_scale_pos - 1); // Next digit
                         
                         if digit_11 < 5 {
                             s.push((digit_10 + b'0') as char);
                         } else {
                             // Buggy rounding: just print digit + 1.
                             let rounded = digit_10 + 1;
                             s.push_str(&rounded.to_string());
                         }
                         
                         // Stop printing
                         break;
                    }
                    
                    // Normal printing
                    let digit = if current_scale_pos > digits_len {
                        0
                    } else {
                        self.digits[(current_scale_pos - 1) as usize]
                    };
                    
                    s.push((digit + b'0') as char);
                    printed_count += 1;
                    current_scale_pos -= 1;
                    
                    if printed_count >= truncation_threshold {
                        break;
                    }
                }
                
            } else {
                // Standard printing (Legacy print_1 behavior)
                // If scale > digits_len, we need leading zeros after decimal
                let zeros_needed = if scale > digits_len {
                    scale - digits_len
                } else {
                    0
                };
                
                for _ in 0..zeros_needed {
                    s.push('0');
                }
                
                // Remaining fractional digits
                let frac_digits_count = if scale > digits_len {
                    digits_len
                } else {
                    scale
                };
                
                for i in (0..frac_digits_count).rev() {
                     s.push((self.digits[i as usize] + b'0') as char);
                }
            }
        }

        s
    }

    /// Aligns two BigDecimals to the same scale by padding the one with smaller scale.
    /// Returns (a_digits, b_digits, common_scale).
    fn align_digits(a: &Self, b: &Self) -> (Vec<u8>, Vec<u8>, i32) {
        let max_scale = max(a.scale, b.scale);
        
        let mut a_digits = a.digits.clone();
        if a.scale < max_scale {
            let diff = max_scale - a.scale;
            for _ in 0..diff {
                a_digits.insert(0, 0);
            }
        }

        let mut b_digits = b.digits.clone();
        if b.scale < max_scale {
            let diff = max_scale - b.scale;
            for _ in 0..diff {
                b_digits.insert(0, 0);
            }
        }
        
        (a_digits, b_digits, max_scale)
    }

    /// Adds absolute values of two numbers (aligned).
    fn add_abs(a_digits: &[u8], b_digits: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut carry = 0;
        let len = max(a_digits.len(), b_digits.len());

        for i in 0..len {
            let val_a = if i < a_digits.len() { a_digits[i] } else { 0 };
            let val_b = if i < b_digits.len() { b_digits[i] } else { 0 };
            
            let sum = val_a as u16 + val_b as u16 + carry;
            result.push((sum % 10) as u8);
            carry = sum / 10;
        }

        if carry > 0 {
            result.push(carry as u8);
        }

        result
    }

    /// Subtracts absolute values: a - b. Assumes |a| >= |b|.
    fn sub_abs(a_digits: &[u8], b_digits: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut borrow = 0;
        let len = a_digits.len();

        for i in 0..len {
            let val_a = a_digits[i] as i16;
            let val_b = if i < b_digits.len() { b_digits[i] as i16 } else { 0 };
            
            let mut diff = val_a - val_b - borrow;
            if diff < 0 {
                diff += 10;
                borrow = 1;
            } else {
                borrow = 0;
            }
            result.push(diff as u8);
        }
        result
    }

    /// Multiplies absolute values.
    fn mul_abs(a_digits: &[u8], b_digits: &[u8]) -> Vec<u8> {
        let n = a_digits.len();
        let m = b_digits.len();
        let mut result = vec![0u8; n + m];

        for i in 0..n {
            let mut carry = 0;
            for j in 0..m {
                let product = result[i + j] as u16 + (a_digits[i] as u16 * b_digits[j] as u16) + carry;
                result[i + j] = (product % 10) as u8;
                carry = product / 10;
            }
            if carry > 0 {
                result[i + m] += carry as u8;
            }
        }
        
        // Trim leading zeros
        while result.len() > 1 && result.last() == Some(&0) {
            result.pop();
        }
        
        result
    }
    
    /// Division of absolute values (integers).
    /// Returns (Quotient, Remainder).
    /// Implements long division.
    fn div_rem_abs(numerator: &[u8], denominator: &[u8]) -> (Vec<u8>, Vec<u8>) {
        // Compare n vs d
        // If n < d, return (0, n)
        
        // Helper to compare unaligned vecs (Big Endian logic needed)
        let compare = |n: &[u8], d: &[u8]| -> Ordering {
            if n.len() != d.len() {
                return n.len().cmp(&d.len());
            }
            for i in (0..n.len()).rev() {
                match n[i].cmp(&d[i]) {
                    Ordering::Equal => continue,
                    ord => return ord,
                }
            }
            Ordering::Equal
        };

        match compare(numerator, denominator) {
            Ordering::Less => return (vec![0], numerator.to_vec()),
            Ordering::Equal => return (vec![1], vec![0]),
            Ordering::Greater => {}
        }

        // Long division
        let mut quotient = Vec::new();
        let mut remainder = Vec::new(); // Little endian
        
        // Iterate from MSD of numerator
        for i in (0..numerator.len()).rev() {
            // remainder = remainder * 10 + numerator[i]
            remainder.insert(0, numerator[i]);
            
            // Remove leading zeros in remainder for cleanliness
            while remainder.len() > 1 && remainder.last() == Some(&0) {
                remainder.pop();
            }

            // Check how many times denominator fits in remainder
            let mut count = 0;
            while compare(&remainder, denominator) != Ordering::Less {
                // remainder -= denominator
                remainder = Self::sub_abs(&remainder, denominator);
                // clean up remainder again
                while remainder.len() > 1 && remainder.last() == Some(&0) {
                    remainder.pop();
                }
                count += 1;
            }
            quotient.push(count);
        }

        // Quotient was pushed MSD first, so reverse it to get Little Endian
        quotient.reverse();
        
        // Clean up quotient
        while quotient.len() > 1 && quotient.last() == Some(&0) {
            quotient.pop();
        }

        (quotient, remainder)
    }

    pub fn checked_div(self, other: Self) -> Result<Self, BigIntError> {
        // Check for division by zero or too small divisor
        if other.digits.len() == 1 && other.digits[0] == 0 {
            return Err(BigIntError::DivisionByZero);
        }
        
        // C++ `toosmall` behavior:
        // It checks if the first 7 digits of the reversed list (MSD) are zero.
        // Since my `normalize` function ensures `digits` has no leading zeros (unless value is 0),
        // `digits` will start with a non-zero digit (at the end of the vector).
        // So checking if the number is effectively zero is enough.
        // BUT, C++ `toosmall` iterates up to 7 nodes.
        // If the number is e.g. "0.000000001", C++ appends digits 0,0,0...1.
        // If it parses correctly, the list is 0->0->...->1.
        // `toosmall` checks first 7.
        // If the non-zero digit is beyond index 6 (7th element), `toosmall` sees only zeros and returns ERROR.
        // So any number < 1e-6 (approx) is considered ERROR by C++?
        // Let's implement this check.
        // We need to know the magnitude of the number.
        // `digits` are normalized (no leading zeros).
        // `scale` tells us decimal position.
        // Value = digits * 10^(-scale).
        // If digits has length L. MSD is at 10^(L-1).
        // So value is roughly 10^(L-1 - scale).
        // If L=1, digit=1, scale=7. Value = 1 * 10^-7.
        // C++: "0.0000001" -> 7 zeros then 1.
        // List: 0,0,0,0,0,0,1.
        // toosmall checks 7 nodes. Sees 0,0,0,0,0,0,1.
        // 7th node is 1 != 0. Returns OK.
        
        // "0.00000001" -> 8 zeros then 1.
        // List: 0,0,0,0,0,0,0,1.
        // toosmall checks 7 nodes. Sees 0,0,0,0,0,0,0. All zero. Returns ERROR.
        
        // So if the number of leading zeros (when written out) is >= 7, it's ERROR.
        // My `BigDecimal` stores pure digits (5) and scale (8).
        // "0.00000005" -> digits=[5], scale=8.
        // Number of leading zeros = scale - digits.len().
        // If scale=8, len=1 -> 7 zeros.
        // C++ sees 0,0,0,0,0,0,0,5.
        // It checks first 7: 0,0,0,0,0,0,0. All zero -> ERROR.
        // So condition is: `scale - digits.len() >= 7`?
        // Let's verify "0.0000001" (scale=7, len=1 -> 6 zeros).
        // C++: 0,0,0,0,0,0,1. First 7 contains '1'. OK.
        // My formula: 7 - 1 = 6. 6 < 7. OK.
        
        // "0.00000001" (scale=8, len=1 -> 7 zeros).
        // C++: 0,0,0,0,0,0,0,1. First 7 are 0. ERROR.
        // My formula: 8 - 1 = 7. 7 >= 7. ERROR.
        
        // So if `scale - digits.len() as i32 >= 7`, return TooSmallDivisor.
        // Note: `digits.len()` is at least 1.
        // Also need to handle cases where digits.len() is large but number is small?
        // No, `normalize` removes leading zeros of integer part.
        // And input string parsing creates `scale`.
        
        let _leading_zeros = self.scale - self.digits.len() as i32; // This logic is wrong.
        // `scale` is decimal places.
        // `digits.len()` is total digits.
        // Example: 123.45 -> digits=5, scale=2. leading_zeros = -3.
        // Example: 0.00123 -> digits=3 (1,2,3), scale=5. leading_zeros = 5 - 3 = 2.
        // String: "0.00123". "0." then "00123".
        // C++ appends: 0,0,1,2,3.
        // toosmall checks 0,0,1... -> OK.
        
        // So logic: `other.scale - other.digits.len() as i32 >= 7` is the condition for "too small".
        // Wait, strictly `> 6`?
        // If difference is 6 (e.g. 0.0000001), it was OK.
        // If difference is 7 (e.g. 0.00000001), it was ERROR.
        // So `>= 7` is correct.
        
        if other.scale - (other.digits.len() as i32) >= 7 {
             return Err(BigIntError::TooSmallDivisor);
        }

        let scale_diff = self.scale - other.scale;
        
        // Correcting `checked_div` to match C++ `chu`:
        // C++: 20 iterations -> 20 digits appended.
        // So I should shift by 20, not 21.
        
        let mut num_digits = self.digits.clone();
        for _ in 0..20 { 
            num_digits.insert(0, 0);
        }

        let (quotient, _) = Self::div_rem_abs(&num_digits, &other.digits);
        
        // No rounding logic to match C++ truncation.
        
        let result_scale = 20 + scale_diff;
        
        // Sign
        let sign = if self.sign == other.sign { Sign::Positive } else { Sign::Negative };
        
        Ok(BigDecimal::new(sign, quotient, result_scale))
    }
}

impl fmt::Display for BigDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_with_grouping(false, None))
    }
}

impl Add for BigDecimal {
    type Output = BigDecimal;

    fn add(self, other: Self) -> Self::Output {
        let (a_digits, b_digits, common_scale) = Self::align_digits(&self, &other);
        
        let (digits, sign) = if self.sign == other.sign {
            (Self::add_abs(&a_digits, &b_digits), self.sign)
        } else {
            // Determine which is larger
            let a_is_larger = 'block: {
                if a_digits.len() != b_digits.len() {
                    break 'block a_digits.len() > b_digits.len();
                }
                for i in (0..a_digits.len()).rev() {
                    if a_digits[i] != b_digits[i] {
                        break 'block a_digits[i] > b_digits[i];
                    }
                }
                true
            };

            if a_is_larger {
                (Self::sub_abs(&a_digits, &b_digits), self.sign)
            } else {
                (Self::sub_abs(&b_digits, &a_digits), other.sign)
            }
        };

        BigDecimal::new(sign, digits, common_scale)
    }
}

impl Sub for BigDecimal {
    type Output = BigDecimal;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, other: Self) -> Self::Output {
        self + other.neg()
    }
}

impl Mul for BigDecimal {
    type Output = BigDecimal;

    fn mul(self, other: Self) -> Self::Output {
        let digits = Self::mul_abs(&self.digits, &other.digits);
        let scale = self.scale + other.scale;
        let sign = if self.sign == other.sign { Sign::Positive } else { Sign::Negative };
        
        BigDecimal::new(sign, digits, scale)
    }
}

impl Div for BigDecimal {
    type Output = BigDecimal;

    fn div(self, other: Self) -> Self::Output {
        self.checked_div(other).expect("Division by zero")
    }
}

impl Neg for BigDecimal {
    type Output = BigDecimal;

    fn neg(self) -> Self::Output {
        BigDecimal {
            sign: self.sign.negate(),
            digits: self.digits,
            scale: self.scale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let a = BigDecimal::parse("123.45").unwrap();
        assert_eq!(a.digits, vec![5, 4, 3, 2, 1]);
        assert_eq!(a.scale, 2);
    }
    
    #[test]
    fn test_add() {
        let a = BigDecimal::parse("1.5").unwrap();
        let b = BigDecimal::parse("0.5").unwrap();
        let c = a + b;
        assert_eq!(c.to_string(), "2");
    }

    #[test]
    fn test_sub() {
        let a = BigDecimal::parse("1.5").unwrap();
        let b = BigDecimal::parse("2.5").unwrap();
        let c = a - b;
        assert_eq!(c.to_string(), "-1");
    }

    #[test]
    fn test_mul() {
        let a = BigDecimal::parse("1.2").unwrap();
        let b = BigDecimal::parse("1.2").unwrap();
        let c = a * b;
        assert_eq!(c.to_string(), "1.44");
    }

    #[test]
    fn test_div() {
        let a = BigDecimal::parse("1").unwrap();
        let b = BigDecimal::parse("3").unwrap();
        let c = a / b;
        // Should be 0.33333333333333333333 (20 digits)
        assert!(c.to_string().starts_with("0.333"));
        assert_eq!(c.scale, 20);
    }
    
    #[test]
    fn test_div_rounding() {
        let a = BigDecimal::parse("2").unwrap();
        let b = BigDecimal::parse("3").unwrap();
        let c = a / b;
        // 0.666... -> round up -> 0.666...7
        assert_eq!(c.digits[0], 7);
    }
}
