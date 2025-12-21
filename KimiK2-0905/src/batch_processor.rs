use std::env;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead};
use high_precision_calculations::BigDecimal;

fn main() {
    // Get input file from command line argument, default to regular test file
    let args: Vec<String> = env::args().collect();
    let input_file = if args.len() > 1 {
        &args[1]
    } else {
        "../llm_crosslang_test_input.txt"
    };

    // Get output file from command line argument, default to out_rust.txt
    let output_file = if args.len() > 2 {
        &args[2]
    } else {
        "out_rust.txt"
    };

    println!("Using input file: {}", input_file);
    println!("Using output file: {}", output_file);

    // Read test input
    let file = File::open(input_file).expect("Cannot open test file");
    let reader = io::BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    // First line is the count
    let count: usize = lines[0].parse().unwrap();

    // Create output file
    let mut output = File::create(output_file).expect("Cannot create output file");

    // Process each operation
    for i in 1..=count.min(lines.len() - 1) {
        let line = &lines[i];
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() != 3 {
            writeln!(output, "ERROR: Invalid format in line {}: {}", i, line).unwrap();
            continue;
        }

        let op = parts[0];
        let num1 = parts[1];
        let num2 = parts[2];

        // Parse numbers
        let a = match BigDecimal::parse(num1) {
            Ok(n) => n,
            Err(_) => {
                writeln!(output, "ERROR: Cannot parse {} in line {}", num1, i).unwrap();
                continue;
            }
        };

        let b = match BigDecimal::parse(num2) {
            Ok(n) => n,
            Err(_) => {
                writeln!(output, "ERROR: Cannot parse {} in line {}", num2, i).unwrap();
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
                writeln!(output, "ERROR: Unknown operation {} in line {}", op, i).unwrap();
                continue;
            }
        };

        match result {
            Ok(val) => {
                // Output the result in simple format (no grouping, no scientific notation)
                writeln!(output, "{}", val).unwrap();
            }
            Err(_) => {
                writeln!(
                    output,
                    "ERROR: Division by zero or other operation error in line {}",
                    i
                )
                .unwrap();
            }
        }
    }

    println!("Test completed! Results written to {}", output_file);
    println!("Processed {} operations", count);
}
