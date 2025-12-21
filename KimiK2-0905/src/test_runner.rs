use std::fs::File;
use std::io::{self, BufRead};
use high_precision_calculations::BigDecimal;

fn main() {
    // Test our implementation with the test input format
    let file = File::open("../llm_crosslang_test_input.txt").expect("Cannot open test file");
    let reader = io::BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    // First line is the count
    let count: usize = lines[0].parse().unwrap();
    println!("Number of operations: {}", count);

    // Process each operation
    for i in 1..=count {
        if i >= lines.len() {
            break;
        }

        let line = &lines[i];
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() != 3 {
            println!("Line {}: Invalid format: {}", i, line);
            continue;
        }

        let op = parts[0];
        let num1 = parts[1];
        let num2 = parts[2];

        println!("Operation {}: {} {} {}", i, op, num1, num2);

        // Parse numbers
        let a = match BigDecimal::parse(num1) {
            Ok(n) => n,
            Err(e) => {
                println!("  Error parsing {}: {:?}", num1, e);
                continue;
            }
        };

        let b = match BigDecimal::parse(num2) {
            Ok(n) => n,
            Err(e) => {
                println!("  Error parsing {}: {:?}", num2, e);
                continue;
            }
        };

        // Perform operation
        let result = match op {
            "+" => a.add(&b),
            "-" => a.sub(&b),
            "*" => a.mul(&b),
            "/" => a.div(&b),
            _ => {
                println!("  Unknown operation: {}", op);
                continue;
            }
        };

        match result {
            Ok(val) => {
                println!("  Result: {}", val);
                println!("  Formatted: {}", val.to_string_with_grouping(true));
            }
            Err(e) => {
                println!("  Operation error: {:?}", e);
            }
        }

        println!();

        if i > 5 {
            // Only test first 5 operations for now
            break;
        }
    }
}
