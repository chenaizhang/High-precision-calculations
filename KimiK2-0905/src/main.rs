//! 高精度十进制数计算器 - 交互式CLI
//!
//! 支持任意精度十进制数的四则运算，可以处理负号、千分位、小数点等格式。

use std::io::{self, BufRead, IsTerminal, Write};
use high_precision_calculations::BigDecimal;

fn main() {
    if !io::stdin().is_terminal() {
        if let Err(err) = run_batch_from_stdin() {
            eprintln!("ERROR: {}", err);
        }
        return;
    }

    println!("=== 高精度十进制数计算器 ===");
    println!("支持操作: + - * /");
    println!("支持格式: 负数(-), 千分位(,), 小数点(.)");
    println!("输入 'quit' 或 'exit' 退出程序");
    println!();

    loop {
        // 获取第一个操作数
        let num1 = match get_input("请输入第一个数字: ") {
            Some(input) => {
                if input.trim() == "quit" || input.trim() == "exit" {
                    println!("感谢使用，再见！");
                    break;
                }
                match BigDecimal::parse(&input) {
                    Ok(n) => n,
                    Err(e) => {
                        println!("错误: {}", e);
                        continue;
                    }
                }
            }
            None => {
                println!("感谢使用，再见！");
                break;
            }
        };

        // 获取操作符
        let op = match get_input("请输入操作符 (+, -, *, /): ") {
            Some(input) => {
                let op = input.trim();
                if op == "quit" || op == "exit" {
                    println!("感谢使用，再见！");
                    break;
                }
                match op {
                    "+" | "-" | "*" | "/" => op.to_string(),
                    _ => {
                        println!("错误: 无效的操作符 '{}'，请使用 +, -, *, /", op);
                        continue;
                    }
                }
            }
            None => {
                println!("感谢使用，再见！");
                break;
            }
        };

        // 获取第二个操作数
        let num2 = match get_input("请输入第二个数字: ") {
            Some(input) => {
                if input.trim() == "quit" || input.trim() == "exit" {
                    println!("感谢使用，再见！");
                    break;
                }
                match BigDecimal::parse(&input) {
                    Ok(n) => n,
                    Err(e) => {
                        println!("错误: {}", e);
                        continue;
                    }
                }
            }
            None => {
                println!("感谢使用，再见！");
                break;
            }
        };

        // 执行运算
        let result = match op.as_str() {
            "+" => num1.add(&num2),
            "-" => num1.sub(&num2),
            "*" => num1.mul(&num2),
            "/" => num1.div(&num2),
            _ => {
                println!("错误: 无效的操作符");
                continue;
            }
        };

        match result {
            Ok(value) => {
                println!("结果: {}", value);
                println!("千分位格式: {}", value.to_string_with_grouping(true));
            }
            Err(e) => {
                println!("计算错误: {}", e);
            }
        }

        println!();
    }
}

fn run_batch_from_stdin() -> Result<(), String> {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin.lock());
    let mut line = String::new();

    let bytes = reader
        .read_line(&mut line)
        .map_err(|e| e.to_string())?;
    if bytes == 0 {
        return Ok(());
    }

    let count: usize = line
        .trim()
        .parse()
        .map_err(|_| "Invalid count line".to_string())?;

    for i in 1..=count {
        line.clear();
        let bytes = reader
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        if bytes == 0 {
            break;
        }

        let trimmed = line.trim();
        let mut parts = trimmed.split_whitespace();
        let (op, num1, num2, extra) =
            (parts.next(), parts.next(), parts.next(), parts.next());
        if op.is_none() || num1.is_none() || num2.is_none() || extra.is_some() {
            println!("ERROR: Invalid format in line {}: {}", i, trimmed);
            continue;
        }

        let op = op.unwrap();
        let num1 = num1.unwrap();
        let num2 = num2.unwrap();

        let a = match BigDecimal::parse(num1) {
            Ok(n) => n,
            Err(_) => {
                println!("ERROR: Cannot parse {} in line {}", num1, i);
                continue;
            }
        };

        let b = match BigDecimal::parse(num2) {
            Ok(n) => n,
            Err(_) => {
                println!("ERROR: Cannot parse {} in line {}", num2, i);
                continue;
            }
        };

        let result = match op {
            "+" => a.add(&b),
            "-" => a.sub(&b),
            "*" => a.mul(&b),
            "/" => a.div(&b),
            _ => {
                println!("ERROR: Unknown operation {} in line {}", op, i);
                continue;
            }
        };

        match result {
            Ok(val) => println!("{}", val),
            Err(_) => {
                println!(
                    "ERROR: Division by zero or other operation error in line {}",
                    i
                );
            }
        }
    }

    Ok(())
}

/// 获取用户输入
fn get_input(prompt: &str) -> Option<String> {
    print!("{}", prompt);
    if io::stdout().flush().is_err() {
        return None;
    }

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) => None,
        Ok(_) => Some(input),
        Err(_) => None,
    }
}
