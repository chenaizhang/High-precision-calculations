mod big_decimal;
use big_decimal::{BigDecimal, BigIntError};
use std::io::{self, Read};

fn main() {
    // Read all input from stdin
    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() {
        return;
    }
    
    let mut tokens = buffer.split_whitespace();
    
    // Read number of operations
    // C++ prompt: "请输入数据数目："
    print!("请输入数据数目：");
    let n_str = match tokens.next() {
        Some(s) => s,
        None => return,
    };
    
    let n = n_str.parse::<usize>().unwrap_or(0);

    for _ in 0..n {
        print!("请输入操作符：");
        let op_str = match tokens.next() {
            Some(s) => s,
            None => break,
        };
        
        let op_char = op_str.chars().next().unwrap_or(' ');
        
        print!("左操作数：");
        let str1 = match tokens.next() {
            Some(s) => s,
            None => break,
        };
        
        print!("右操作数：");
        let str2 = match tokens.next() {
            Some(s) => s,
            None => break,
        };

        match perform_operation(op_char, str1, str2) {
            Ok(result) => {
                // Formatting with grouping (commas) as per C++ behavior
                // C++ uses print_1 for +/- (unlimited decimal places?)
                // C++ uses print_2 for * / (limited to 10 decimal places with buggy rounding)
                let limit = match op_char {
                    '*' | '/' => Some(10),
                    _ => None,
                };
                println!("{}", result.to_string_with_grouping(true, limit));
            },
            Err(_) => println!("ERROR"),
        }
    }
}

fn perform_operation(op: char, s1: &str, s2: &str) -> Result<BigDecimal, BigIntError> {
    let num1 = BigDecimal::parse(s1)?;
    let num2 = BigDecimal::parse(s2)?;

    match op {
        '+' => Ok(num1 + num2),
        '-' => Ok(num1 - num2),
        '*' => Ok(num1 * num2),
        '/' => num1.checked_div(num2),
        _ => Err(BigIntError::ParseError(format!("Unknown operator: {}", op))),
    }
}
