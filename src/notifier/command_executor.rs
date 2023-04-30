use std::io::{self, prelude::*};
use std::process::{Command, Stdio};

use super::notifier_base::Error;
use super::Message;
use super::NotifierStrategy;

pub struct CommandExecutor {
    command: String,
    args: Vec<String>,
    pipe: bool,
}

impl CommandExecutor {
    pub fn new(command: &str, args: Vec<String>, pipe: bool) -> Self {
        Self {
            command: command.to_owned(),
            args,
            pipe,
        }
    }
}

fn print_result(stdout: &[u8], stderr: &[u8]) -> Result<(), Error> {
    io::stdout()
        .write_all(stdout)
        .map_err(|e| Error::FailedToPostMessage(e.to_string()))?;
    io::stderr()
        .write_all(stderr)
        .map_err(|e| Error::FailedToPostMessage(e.to_string()))?;

    Ok(())
}

impl NotifierStrategy for CommandExecutor {
    fn notify(&self, message: &Message) -> Result<(), Error> {
        let modified_args = self
            .args
            .iter()
            .map(|arg| if arg == "{msg}" { message.body() } else { arg });

        if self.pipe {
            let mut child = Command::new(&self.command)
                .args(modified_args)
                .stdin(Stdio::piped())
                .spawn()
                .map_err(|e| Error::FailedToPostMessage(e.to_string()))?;

            // Write a message into stdin
            let child_stdin = child.stdin.as_mut().ok_or_else(|| {
                Error::FailedToPostMessage("failed to open stdin of child process".to_string())
            })?;
            child_stdin
                .write_all(message.body().as_bytes())
                .map_err(|_| {
                    Error::FailedToPostMessage(
                        "failed to write to stdin of child process".to_string(),
                    )
                })?;
            #[allow(clippy::drop_ref)]
            drop(child_stdin); // Write EOF

            let output = child.wait_with_output().map_err(|_| {
                Error::FailedToPostMessage("failed to get output of child process".to_string())
            })?;

            print_result(&output.stdout, &output.stderr)?;
        } else {
            let output = Command::new(&self.command)
                .args(modified_args)
                .output()
                .map_err(|_| {
                    Error::FailedToPostMessage("failed to execute child process".to_string())
                })?;

            print_result(&output.stdout, &output.stderr)?;
        }

        Ok(())
    }
}
