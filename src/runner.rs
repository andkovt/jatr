use crate::tasks::{Action, ActionCommand, Task, TaskFile};
use crate::templater::{TemplateError, Templater};
use crate::{output};
use log::{debug, error, log_enabled, Level};
use std::process::ExitStatus;
use std::{io, process};

#[derive(Debug)]
#[allow(unused)]
pub enum RunActionError {
    IO(io::Error),
    TaskNotFound(String),
    TemplatingError(TemplateError)
}

#[derive(Debug)]
pub struct RunResult {
    last_action: ActionStatus
}

#[derive(Debug)]
pub struct ActionStatus {
    #[allow(unused)]
    action: Action,
    success: bool,
    cmd_status: Option<CmdStatus>
}

#[derive(Debug)]
pub struct CmdStatus {
    #[allow(unused)]
    pub cmd: String,
    pub exit_status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

pub struct ActionRunner {
    task_name: String,
    actions: Vec<Action>,
    directory: String,
    templater: Templater,
}

impl ActionRunner {
    pub fn for_task(
        name: &str,
        task: &Task,
        directory: &str,
        templater: Templater
    ) -> Self {
        Self {
            task_name: String::from(name),
            actions: task.actions.to_vec(),
            directory: String::from(directory),
            templater
        }
    }

    pub fn run(&mut self, tasks: &TaskFile) -> Result<RunResult, RunActionError> {
        let mut latest_result = ActionStatus { action: Action::Noop, cmd_status: None, success: true };

        match self.templater.resolve_variables(|_name, shell, val| {
            Self::run_cmd(shell_or_default(shell).as_str(), val, &self.directory, false, true)
                .map_err(TemplateError::VariableResolveIO)
        }) {
            Ok(_) => {}
            Err(e) => {
                return Err(RunActionError::TemplatingError(e));
            }
        }

        for action in &self.actions {
            let templated_action = self.templater.template(action)
                .map_err(RunActionError::TemplatingError)?;

            latest_result = match self.run_action(&templated_action, tasks) {
                Ok(result) => result,
                Err(e) => {
                    error!("Failed to run command {:?}", e);
                    return Err(e);
                }
            };

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
                        );
                        break;
                    }

                    if cmd_status.stdout.trim().to_lowercase() == "false" {
                        output::if_execution(
                            &self.task_name,
                            &cmd.command,
                            false,
                        );

                        break;
                    }

                    output::if_execution(
                        &self.task_name,
                        &cmd.command,
                        true,
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

        Ok(RunResult { last_action: latest_result })
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

                let mut runner = ActionRunner::for_task(
                    &call.name,
                    task,
                    &*self.directory,
                    Templater::for_task(task, tasks)
                );

                let test = runner.run(tasks);

                Ok(test?.last_action)
            }
        }
    }

    fn run_action_command(&self, ac: &ActionCommand, tty: bool, silent: bool) -> Result<CmdStatus, io::Error> {
        Self::run_cmd(shell_or_default(ac.shell.as_ref()).as_str(), &ac.command, &self.directory, tty, silent)
    }

    fn run_cmd(shell: &str, cmd: &str, work_dir: &str, tty: bool, silent: bool) -> Result<CmdStatus, io::Error> {
        let mut proc_cmd = process::Command::new(shell);
        proc_cmd.arg("-c").arg(cmd)
            .current_dir(work_dir);

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

        if !stdout.is_empty() && !tty && !silent {
            println!("{}", stdout.trim());
        }

        debug!("Command '{}' exited with status code '{}'", cmd, status.code().unwrap_or(-1));
        debug!("stdout: {}, stderr: {}", stdout.trim(), stderr.trim());

        Ok(CmdStatus { cmd: String::from(cmd), exit_status: status, stdout, stderr })
    }
}

pub fn shell_or_default(shell: Option<&String>) -> String {
    match shell {
        Some(shell) => String::from(shell),
        None => String::from("sh"),
    }
}