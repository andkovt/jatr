use std::{io, process};
use std::process::ExitStatus;
use std::slice::Iter;
use log::{debug, error, info, log_enabled, warn, Level};
use tera::{Context, Tera, Value};
use crate::{output, CommandStatus};
use crate::tasks::{Action, ActionCommand, Task, TaskFile};
use crate::templater::{TemplateError, Templater};

#[derive(Debug)]
pub enum RunActionError {
    IO(io::Error),
    TaskNotFound(String),
    TemplatingError(TemplateError)
}

#[derive(Debug)]
pub struct RunResult {
    success: bool,
    last_action: ActionStatus
}

#[derive(Debug)]
pub struct ActionStatus {
    action: Action,
    success: bool,
    cmd_status: Option<CmdStatus>
}

#[derive(Debug)]
pub struct CmdStatus {
    cmd: String,
    exit_status: ExitStatus,
    stdout: String,
    stderr: String,
}

pub struct ActionRunner<'a> {
    task_name: String,
    task: &'a Task,
    actions: Vec<Action>,
    directory: String,
    templater: Templater,
}

impl<'a> ActionRunner<'a> {
    pub fn for_task(
        name: &str,
        task: &'a Task,
        directory: &str,
        templater: Templater
    ) -> Self {
        Self {
            task_name: String::from(name),
            actions: task.actions.to_vec(),
            directory: String::from(directory),
            task,
            templater
        }
    }

    pub fn run(&self, tasks: &TaskFile) -> Result<RunResult, RunActionError> {
        let mut latest_result = ActionStatus { action: Action::Noop, cmd_status: None, success: true };
        let mut success = true;

        for action in &self.actions {
            let templated_action = self.templater.template(action)
                .map_err(|e| RunActionError::TemplatingError(e))?;

            latest_result = match self.run_action(&templated_action, tasks) {
                Ok(result) => result,
                Err(e) => {
                    error!("Failed to run command {:?}", e);
                    return Err(e);
                }
            };

            success = latest_result.success;

            let Some(cmd_status) = &latest_result.cmd_status else {
                continue;
            };

            match templated_action {
                Action::If(cmd) => {
                    if !cmd_status.exit_status.success() {
                        if cmd_status.exit_status.code().unwrap_or(-1) != 1 {
                            error!("Task {}: if {} resulted in status code '{}'", self.task_name, cmd.command, cmd_status.exit_status);
                            error!("stderr: '{}'", cmd_status.stderr);
                        }

                        output::if_execution(
                            &self.task_name,
                            &cmd.command,
                            false,
                            &format!("status code({})", cmd_status.exit_status.code().unwrap_or(-1))
                        );
                        break;
                    }

                    if cmd_status.stdout.trim().to_lowercase() == "false" {
                        output::if_execution(
                            &self.task_name,
                            &cmd.command,
                            false,
                            &format!("falsely output({})", cmd_status.stdout.trim())
                        );

                        break;
                    }

                    output::if_execution(
                        &self.task_name,
                        &cmd.command,
                        true,
                        ""
                    );

                    continue
                }
                _ => {
                    if !latest_result.success {
                        break
                    }
                }
            }
        }

        Ok(RunResult {
            success,
            last_action: latest_result,
        })
    }

    fn run_action(&self, action: &Action, tasks: &TaskFile) -> Result<ActionStatus, RunActionError> {
        match action {
            Action::Noop => {
                debug!("Noop action");
                Ok(ActionStatus{action: action.clone(), cmd_status: None, success: true})
            },
            Action::Command(cmd) => {
                output::cmd_execution(&self.task_name, &cmd.command);
                debug!("Task {}: Running cmd action '{}'", self.task_name, cmd.command);
                // info!("Task {}: {}", self.task, cmd.command);

                self.run_action_command(cmd, true, false)
                    .map_err(|e| RunActionError::IO(e))
                    .map(|s| ActionStatus{action: action.clone(), success: s.exit_status.success(), cmd_status: Some(s)})
            },
            Action::If(cmd) => {
                debug!("Task {}: Running if action '{}'", self.task_name, cmd.command);

                self.run_action_command(cmd, false, true)
                    .map_err(|e| RunActionError::IO(e))
                    .map(|s| {
                        let was_success = s.exit_status.code().unwrap_or(-1) < 2;
                        ActionStatus{
                            action: action.clone(),
                            success: was_success,
                            cmd_status: Some(s)}
                    })
            }
            Action::Task(call) => {
                debug!("Task {}: Running task action '{}'", self.task_name, call.name);

                let Some(task) = tasks.tasks.get(&call.name) else {
                    error!("Unable to find task '{}'", call.name);

                    return Err(RunActionError::TaskNotFound(String::from(&call.name)))
                };

                let runner = ActionRunner::for_task(
                    &call.name,
                    task,
                    &*self.directory,
                    Templater::for_task(task, tasks).unwrap()
                );

                let test = runner.run(tasks);

                Ok(test?.last_action)
            }
        }
    }

    fn run_action_command(&self, ac: &ActionCommand, tty: bool, silent: bool) -> Result<CmdStatus, io::Error> {
        let shell = match &ac.shell {
            Some(shell) => shell,
            None => &String::from("sh"),
        };

        self.run_cmd(shell, &ac.command, tty, silent)
    }

    fn run_cmd(&self, shell: &str, cmd: &str, tty: bool, silent: bool) -> Result<CmdStatus, io::Error> {
        let mut proc_cmd = process::Command::new(shell);
        proc_cmd.arg("-c").arg(cmd)
            .current_dir(&self.directory);

        let output = match tty {
            true => {
                let child = proc_cmd.spawn()?;
                child.wait_with_output()?
            },
            false => {
                proc_cmd.output()?
            }
        };

        let stdout = String::from_utf8(output.stdout).unwrap_or_else(|e| {
            debug!("Unable to parse stdout as utf8 string: {:?}", e);
            String::new()
        });

        let stderr = String::from_utf8(output.stderr).unwrap_or_else(|e| {
            debug!("Unable to parse stderr as utf8 string: {:?}", e);
            String::new()
        });

        let status = output.status;
        if !status.success() && !silent {
            error!("Command '{}' exited with status code '{}'", cmd, status.code().unwrap_or(-1));
            if log_enabled!(Level::Debug) && stderr.len() > 0 {
                error!("{}", stderr)
            }

            return Ok(CmdStatus { cmd: String::from(cmd) , exit_status: status, stdout, stderr });
        }

        if stdout.len() > 0 {
            println!("{}", stdout.trim());
        }

        debug!("Command '{}' exited with status code '{}'", cmd, status.code().unwrap_or(-1));
        debug!("stdout: {}, stderr: {}", stdout.trim(), stderr.trim());

        Ok(CmdStatus { cmd: String::from(cmd), exit_status: status, stdout, stderr })
    }
}