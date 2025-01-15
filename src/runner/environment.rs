use crate::runner::Result;
use crate::S;
use log::debug;
use std::process::{Command, ExitStatus};

#[derive(Debug)]
pub struct ExecuteResult {
    pub cmd: String,
    pub exit_status: ExitStatus,
    pub stdout: String,

    #[allow(dead_code)]
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct RunnerEnvironment {
    default_shell: String,
    working_dir: String,
}

impl Default for RunnerEnvironment {
    fn default() -> Self {
        Self {
            default_shell: S!("sh"),
            working_dir: String::from("."),
        }
    }
}

impl RunnerEnvironment {
    pub fn execute(
        &self,
        shell_command: &str,
        shell: Option<&str>,
        tty: bool,
    ) -> Result<ExecuteResult> {
        let mut cmd = self.create_cmd(shell, shell_command);

        let output = match tty {
            true => cmd.spawn()?.wait_with_output()?,
            false => cmd.output()?,
        };

        debug!(
            "Executed: {}. Exit Code: {}. Stdout: '{}' Stderr: '{}'",
            shell_command,
            output.status,
            String::from_utf8(output.stdout.clone()).unwrap(),
            String::from_utf8(output.stderr.clone()).unwrap()
        );

        Ok(ExecuteResult {
            cmd: String::from(shell_command),
            exit_status: output.status,
            stdout: String::from_utf8(output.stdout).unwrap(),
            stderr: String::from_utf8(output.stderr).unwrap(),
        })
    }

    fn create_cmd(&self, shell: Option<&str>, shell_command: &str) -> Command {
        let mut cmd = Command::new(match shell {
            Some(s) => String::from(s),
            None => self.default_shell.clone(),
        });

        cmd.current_dir(self.working_dir.clone());
        cmd.arg("-c").arg(shell_command);

        cmd
    }

    pub fn work_dir(&mut self, dir: &str) {
        self.working_dir = String::from(dir);
    }
}

impl Default for ExecuteResult {
    fn default() -> Self {
        Self {
            cmd: String::new(),
            exit_status: ExitStatus::default(),
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}
