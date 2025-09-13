use predicates::prelude::*;
use std::error::Error;

#[test]
fn add_default_precision() -> Result<(), Box<dyn Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("mycalc")?;
    cmd.args(["add", "1", "2", "3"]) // 1 + 2 + 3 = 6
        .assert()
        .success()
        .stdout(predicate::str::starts_with("6.00"));
    Ok(())
}

#[test]
fn add_custom_precision() -> Result<(), Box<dyn Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("mycalc")?;
    cmd.args(["-p", "3", "add", "1", "2", "3"]) // 6 con 3 decimales
        .assert()
        .success()
        .stdout(predicate::str::starts_with("6.000"));
    Ok(())
}

#[test]
fn sub_left_associative() -> Result<(), Box<dyn Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("mycalc")?;
    cmd.args(["sub", "10", "3", "2"]) // (10 - 3 - 2) = 5
        .assert()
        .success()
        .stdout(predicate::str::starts_with("5.00"));
    Ok(())
}

#[test]
fn mul_basic() -> Result<(), Box<dyn Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("mycalc")?;
    cmd.args(["mul", "2", "3", "4"]) // 24
        .assert()
        .success()
        .stdout(predicate::str::starts_with("24.00"));
    Ok(())
}

#[test]
fn div_ok() -> Result<(), Box<dyn Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("mycalc")?;
    cmd.args(["div", "20", "2", "5"]) // (20/2/5)=2
        .assert()
        .success()
        .stdout(predicate::str::starts_with("2.00"));
    Ok(())
}

#[test]
fn div_by_zero_errors() -> Result<(), Box<dyn Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("mycalc")?;
    cmd.args(["div", "10", "0"]) // error
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: división por cero"));
    Ok(())
}
