use std::io;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;

use crate::Terminal;

#[derive(Debug)]
pub(crate) struct IssuedCommand {
    pub(crate) args: String,
    pub(crate) child: Child,
    pub(crate) output: Vec<u8>,
    pub(crate) finish_acked: bool,
}

impl IssuedCommand {
    pub(crate) fn spawn(input: &[u8], mut command: Command) -> Result<IssuedCommand, io::Error> {
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn()?;

        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap_or_else(|| panic!("No stdin for process"))
            .write_all(input)
            .unwrap_or_else(|_| panic!("Error writing to stdin"));

        let issued_command = IssuedCommand {
            args: format_command(&command),
            child,
            output: vec![],
            finish_acked: false,
        };
        Ok(issued_command)
    }

    pub(crate) fn spawn_in_subscreen(
        terminal: &mut Terminal,
        mut command: Command,
    ) -> Result<IssuedCommand, io::Error> {
        command.stdin(Stdio::piped());
        let mut child = command.spawn()?;

        child.wait()?;

        terminal.hide_cursor()?;
        terminal.clear()?;

        let issued_command = IssuedCommand {
            args: format_command(&command),
            child,
            output: vec![],
            finish_acked: false,
        };
        Ok(issued_command)
    }

    pub(crate) fn read_command_output_to_buffer(&mut self) {
        if let Some(stderr) = self.child.stderr.as_mut() {
            let mut buffer = [0; 256];

            use std::io::Read;
            let read = stderr
                .read(&mut buffer)
                .expect("Error reading child stderr");

            self.output.extend(&buffer[..read]);
        }
    }

    pub(crate) fn is_running(&mut self) -> bool {
        !self.child.try_wait().is_ok_and(|status| status.is_some())
    }

    pub(crate) fn just_finished(&mut self) -> bool {
        if self.finish_acked {
            return false;
        }

        let Some(_status) = self.child.try_wait().expect("Error awaiting child") else {
            return false;
        };

        self.finish_acked = true;
        true
    }
}

fn format_command(cmd: &Command) -> String {
    let command_display = format!(
        "{} {}",
        cmd.get_program().to_string_lossy(),
        cmd.get_args()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );
    command_display
}
