use std::process::Command;

#[test]
fn test_lox_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_rlox"))
        .arg("test.lox")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        include_str!("expected.txt")
    );
}
