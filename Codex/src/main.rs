use std::io::{self, Read, Write};

use high_precision_calculations::{BigDecimal, LegacyFormat};

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return;
    }
    let mut iter = input.split_whitespace();

    print!("请输入数据数目：");
    let _ = io::stdout().flush();
    let n: usize = match iter.next() {
        Some(token) => token.parse().unwrap_or(0),
        None => return,
    };

    for _ in 0..n {
        print!("请输入操作符：");
        let _ = io::stdout().flush();
        let op = match iter.next() {
            Some(token) => token.chars().next().unwrap_or(' '),
            None => return,
        };

        print!("左操作数：");
        let _ = io::stdout().flush();
        let left_raw = match iter.next() {
            Some(token) => token,
            None => return,
        };

        print!("右操作数：");
        let _ = io::stdout().flush();
        let right_raw = match iter.next() {
            Some(token) => token,
            None => return,
        };

        let left = BigDecimal::parse(left_raw);
        let right = BigDecimal::parse(right_raw);

        let output = match (op, left, right) {
            ('+', Ok(a), Ok(b)) => a.add(&b).to_string_legacy(LegacyFormat::AddSub),
            ('-', Ok(a), Ok(b)) => a.sub(&b).to_string_legacy(LegacyFormat::AddSub),
            ('*', Ok(a), Ok(b)) => a.mul(&b).to_string_legacy(LegacyFormat::MulDiv),
            ('/', Ok(a), Ok(b)) => match a.div(&b) {
                Ok(value) => value.to_string_legacy(LegacyFormat::MulDiv),
                Err(_) => "ERROR".to_string(),
            },
            _ => "ERROR".to_string(),
        };

        println!("{}", output);
    }
}
