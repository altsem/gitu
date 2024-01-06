use std::io::Write;
use std::process::Stdio;

use std::process::Command;

pub(crate) fn run(program: &str, args: &[&str]) -> (String, String) {
    let output = Command::new(program)
        .args(args)
        .output()
        .unwrap_or_else(|_| panic!("Couldn't execute '{}'", program));

    (
        String::from_utf8(output.stdout)
            .unwrap_or_else(|_| panic!("Couldn't read stdout of '{}'", program)),
        String::from_utf8(output.stderr)
            .unwrap_or_else(|_| panic!("Couldn't read stderr of '{}'", program)),
    )
}

pub(crate) fn pipe(input: &[u8], program: &str, args: &[&str]) -> String {
    let mut command = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("Error executing '{}'", program));
    command
        .stdin
        .take()
        .unwrap_or_else(|| panic!("No stdin for {} process", program))
        .write_all(input)
        .unwrap_or_else(|_| panic!("Error writing to '{}' stdin", program));
    String::from_utf8(
        command
            .wait_with_output()
            .unwrap_or_else(|_| panic!("Error writing {} output", program))
            .stdout,
    )
    .unwrap()
}
