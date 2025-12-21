use high_precision_calculations::{BigDecimal, LegacyFormat};

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
