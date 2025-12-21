use high_precision_calculations::{BigDecimal, LegacyFormat};
use std::io::IsTerminal;
use std::io::{self, Read, Write};

fn main() {
    if io::stdin().is_terminal() {
        run_interactive();
    } else {
        run_batch();
    }
}

fn run_interactive() {
    println!("高精度十进制数计算器");
    println!("====================");

    loop {
        println!("\n请输入操作类型 (+, -, *, /) 或 'quit' 退出:");
        
        let mut op = String::new();
        if io::stdin().read_line(&mut op).expect("读取输入失败") == 0 {
            break;
        }
        let op = op.trim();

        if op.eq_ignore_ascii_case("quit") {
            break;
        }

        if !["+", "-", "*", "/"].contains(&op) {
            println!("无效的操作符，请重新输入");
            continue;
        }

        println!("请输入左操作数:");
        let mut left_str = String::new();
        if io::stdin()
            .read_line(&mut left_str)
            .expect("读取输入失败")
            == 0
        {
            break;
        }
        let left_str = left_str.trim();

        println!("请输入右操作数:");
        let mut right_str = String::new();
        if io::stdin()
            .read_line(&mut right_str)
            .expect("读取输入失败")
            == 0
        {
            break;
        }
        let right_str = right_str.trim();

        let op_char = op.chars().next().unwrap_or(' ');
        let output = compute_output(op_char, left_str, right_str);
        println!("结果: {}", output);
    }

    println!("感谢使用！");
}

fn run_batch() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let mut out = io::BufWriter::new(io::stdout());
    let _ = write!(out, "请输入数据数目：");
    let mut iter = input.split_whitespace();
    let n: usize = match iter.next() {
        Some(token) => token.parse().unwrap_or(0),
        None => return,
    };

    for _ in 0..n {
        let _ = write!(out, "请输入操作符：");
        let op = match iter.next() {
            Some(token) => token.chars().next().unwrap_or(' '),
            None => return,
        };

        let _ = write!(out, "左操作数：");
        let left_raw = match iter.next() {
            Some(token) => token,
            None => return,
        };

        let _ = write!(out, "右操作数：");
        let right_raw = match iter.next() {
            Some(token) => token,
            None => return,
        };

        let output = compute_output(op, left_raw, right_raw);
        let _ = writeln!(out, "{}", output);
    }
}

fn compute_output(op: char, left_raw: &str, right_raw: &str) -> String {
    let left = BigDecimal::parse(left_raw);
    let right = BigDecimal::parse(right_raw);

    match (op, left, right) {
        ('+', Ok(a), Ok(b)) => a.add(&b).to_string_legacy(LegacyFormat::AddSub),
        ('-', Ok(a), Ok(b)) => a.sub(&b).to_string_legacy(LegacyFormat::AddSub),
        ('*', Ok(a), Ok(b)) => a.mul(&b).to_string_legacy(LegacyFormat::MulDiv),
        ('/', Ok(a), Ok(b)) => match a.div(&b) {
            Ok(value) => value.to_string_legacy(LegacyFormat::MulDiv),
            Err(_) => "ERROR".to_string(),
        },
        _ => "ERROR".to_string(),
    }
}
