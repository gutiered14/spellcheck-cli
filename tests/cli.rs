use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn piped_text_reports_issues_and_exit_status() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_spellcheck"))
        .arg("--no-color")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"Hello, wurld! The the  cat")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("wurld") && stdout.contains("world"));
    assert!(stdout.contains("Repeated") && stdout.contains("Multiple spaces"));
    assert!(!stdout.contains('\x1b'));
}
