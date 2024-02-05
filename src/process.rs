use std::io::Write;
use std::process::Stdio;

use std::process::Command;

pub(crate) fn pipe(input: &[u8], cmd: &[&str]) -> (String, String) {
    let mut command = Command::new(cmd[0])
        .args(&cmd[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("Error executing '{:?}'", cmd));
    command
        .stdin
        .take()
        .unwrap_or_else(|| panic!("No stdin for {:?} process", cmd))
        .write_all(input)
        .unwrap_or_else(|_| panic!("Error writing to '{:?}' stdin", cmd));
    let output = command
        .wait_with_output()
        .unwrap_or_else(|_| panic!("Error writing {:?} output", cmd));

    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}
