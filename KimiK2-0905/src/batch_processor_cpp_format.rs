use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use high_precision_calculations::BigDecimal;

fn main() {
    // Open the test input file
    let file = File::open("../llm_crosslang_test_input.txt").expect("Cannot open test file");
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    // Create output file
    let mut output = File::create("out_rust_cpp_format.txt").expect("Cannot create output file");

    // First line is the number of operations
    let n: i32 = lines[0].parse().expect("Invalid number of operations");

    // Write the initial prompt (no newline, same as C++)
    write!(output, "请输入数据数目：").unwrap();

    // Process each operation
    for i in 1..=n as usize {
        if i >= lines.len() {
            break;
        }

        let line = &lines[i];
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() != 3 {
            writeln!(output, "请输入操作符：左操作数：右操作数：ERROR").unwrap();
            continue;
        }

        let op = parts[0];
        let num1 = parts[1];
        let num2 = parts[2];

        // Parse numbers
        let a = match BigDecimal::parse(num1) {
            Ok(n) => n,
            Err(_) => {
                writeln!(output, "请输入操作符：左操作数：右操作数：ERROR").unwrap();
                continue;
            }
        };

        let b = match BigDecimal::parse(num2) {
            Ok(n) => n,
            Err(_) => {
                writeln!(output, "请输入操作符：左操作数：右操作数：ERROR").unwrap();
                continue;
            }
        };

        // Perform operation
        let result = match op {
            "+" => a.add(&b),
            "-" => a.sub(&b),
            "*" => a.mul(&b),
            "/" => {
                if b.is_zero() {
                    Err(high_precision_calculations::BigDecimalError::DivisionByZero)
                } else {
                    a.div(&b)
                }
            }
            _ => Err(high_precision_calculations::BigDecimalError::InvalidInput(
                "Unknown operation".to_string(),
            )),
        };

        match result {
            Ok(val) => {
                // Format output to match C++ format
                let result_str = format_result_cpp_style(&val);
                writeln!(output, "请输入操作符：左操作数：右操作数：{}", result_str).unwrap();
            }
            Err(_) => {
                writeln!(output, "请输入操作符：左操作数：右操作数：ERROR").unwrap();
            }
        }
    }

    println!("Test completed! Results written to out_rust_cpp_format.txt");
    println!("Processed {} operations", n);
}

fn format_result_cpp_style(num: &BigDecimal) -> String {
    let s = num.to_string();

    // Handle negative numbers
    let (is_negative, digits) = if s.starts_with('-') {
        (true, &s[1..])
    } else {
        (false, s.as_str())
    };

    // Split into integer and decimal parts
    let parts: Vec<&str> = digits.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = if parts.len() > 1 { parts[1] } else { "" };

    // Format integer part with commas every 3 digits from right
    let mut formatted_int = String::new();
    let int_len = integer_part.len();

    for (i, ch) in integer_part.chars().enumerate() {
        if i > 0 && (int_len - i) % 3 == 0 {
            formatted_int.push(',');
        }
        formatted_int.push(ch);
    }

    // Combine with decimal part
    let mut result = if is_negative {
        "-".to_string()
    } else {
        String::new()
    };
    result.push_str(&formatted_int);

    if !decimal_part.is_empty() {
        result.push('.');
        result.push_str(decimal_part);
    }

    result
}
