pub mod environment;
mod output;
pub mod templating;

use crate::tasks::{Action, ActionCommand, Task, TaskFile, Variable, VariableValue};
use environment::{ExecuteResult, RunnerEnvironment};
use log::{debug, error};
use output::Output;
use serde_yaml::Value;
use std::result;
use templating::Templating;

pub type Result<T> = result::Result<T, RunnerError>;

#[derive(Debug)]
#[allow(dead_code)]
pub struct RunnerError(String, RunnerErrorData);

#[derive(Debug)]
#[allow(dead_code)]
pub enum RunnerErrorData {
    TemplateError(tera::Error),
    VariableResolveError { variable: String },
    Io(std::io::Error),
    TaskNotFound,
}

pub enum RunnerResult {
    Success,
    Skipped,
    Failure,
}

#[derive(Debug)]
pub struct ActionResult {
    last_command: ExecuteResult,
    break_execution: bool,
    failed: bool,
}

pub struct Runner<'a> {
    task_file: &'a TaskFile,
    templating: Templating,
    environment: RunnerEnvironment,
    output: Output,
}

impl<'a> Runner<'a> {
    pub fn for_taskfile(task_file: &'a TaskFile, environment: RunnerEnvironment) -> Self {
        Self {
            task_file,
            templating: Templating::default(),
            environment,
            output: Output::for_task(""),
        }
    }

    pub fn run(&mut self, task: &Task) -> Result<RunnerResult> {
        self.output = Output::for_task(task.name.as_str());

        debug!("Running task: {}", task.name);
        self.resolve_variables(task)?;

        for action in task.actions.iter() {
            let action_result = self.run_action(action, false)?;

            if action_result.failed {
                error!("Action failed: {:?}", action_result);
                return Ok(RunnerResult::Failure);
            }

            if action_result.break_execution {
                return Ok(RunnerResult::Skipped);
            }
        }

        Ok(RunnerResult::Success)
    }

    fn run_action(&self, action: &Action, silent: bool) -> Result<ActionResult> {
        match action {
            Action::Command(cmd) => {
                let result = self.run_action_command(cmd, action, silent)?;
                let break_execution = !result.exit_status.success();

                return Ok(ActionResult {
                    last_command: result,
                    break_execution,
                    failed: break_execution,
                });
            }
            Action::If(cmd) => {
                let result = self.run_action_command(cmd, action, silent)?;
                let mut break_execution = !result.exit_status.success();

                if !break_execution && result.stdout.trim() == "false" {
                    break_execution = true;
                }

                self.output
                    .if_execution(result.cmd.as_str(), !break_execution);

                return Ok(ActionResult {
                    last_command: result,
                    break_execution,
                    failed: false,
                });
            }
            Action::Task(call) => {
                let Some(task) = self.task_file.tasks.get(call.name.as_str()) else {
                    return Err(RunnerError(
                        format!("Task '{}' not found", call.name),
                        RunnerErrorData::TaskNotFound,
                    ));
                };

                let mut runner = Runner::for_taskfile(self.task_file, self.environment.clone());
                let result = runner.run(task)?;

                let failed = match result {
                    RunnerResult::Failure => true,
                    _ => false,
                };

                return Ok(ActionResult {
                    last_command: ExecuteResult::default(),
                    break_execution: failed,
                    failed,
                });
            }
            Action::Noop => {}
        }

        Ok(ActionResult {
            last_command: ExecuteResult::default(),
            break_execution: false,
            failed: false,
        })
    }

    fn run_action_command(
        &self,
        cmd: &ActionCommand,
        action: &Action,
        silent: bool,
    ) -> Result<ExecuteResult> {
        let templated_command = self.templating.process(&cmd.command)?;

        if !silent {
            match action {
                Action::Command(_) => {
                    self.output.cmd_execution(&templated_command);
                }
                _ => {}
            }
        }

        let result = self
            .environment
            .execute(templated_command.as_str(), None, cmd.tty)?;

        Ok(result)
    }

    fn resolve_variables(&mut self, task: &Task) -> Result<()> {
        for var in self.task_file.variables.iter() {
            debug!("Resolving global ariable: {:?}", var);

            let value = self.resolve_variable(var)?;
            self.templating.add_variable(var.name.as_str(), value);
        }

        for var in task.variables.iter() {
            debug!("Resolving task variable: {:?}", var);

            let value = self.resolve_variable(var)?;
            self.templating.add_variable(var.name.as_str(), value);
        }
        Ok(())
    }

    fn resolve_variable(&self, variable: &Variable) -> Result<Value> {
        match &variable.value {
            VariableValue::Static(s) => match s {
                Value::String(s) => {
                    let templated = self.templating.process(s)?;
                    Ok(Value::String(templated))
                }
                val => Ok(val.clone()),
            },
            VariableValue::Action(action) => {
                Ok(self.resolve_variable_action(variable.name.as_str(), action)?)
            }
        }
    }

    fn resolve_variable_action(&self, var_name: &str, action: &Action) -> Result<Value> {
        let Action::Command(cmd) = action else {
            return Err(RunnerError(
                format!("Invalid action type for variable '{}'", var_name),
                RunnerErrorData::VariableResolveError {
                    variable: String::from(var_name),
                },
            ));
        };

        let templated_command = self.templating.process(&cmd.command)?;
        let mut cmd_clone = cmd.clone();
        cmd_clone.command = templated_command;

        let action_result = self.run_action(&Action::Command(cmd_clone), true)?;
        if !action_result.last_command.exit_status.success() {
            return Err(RunnerError(
                format!(
                    "Error resolving variable '{}'. Command '{}' exit code: {}",
                    var_name,
                    action_result.last_command.cmd,
                    action_result.last_command.exit_status.code().unwrap()
                ),
                RunnerErrorData::VariableResolveError {
                    variable: String::from(var_name),
                },
            ));
        }

        Ok(Value::String(action_result.last_command.stdout))
    }
}

impl From<std::io::Error> for RunnerError {
    fn from(e: std::io::Error) -> Self {
        RunnerError(String::from("IO Error"), RunnerErrorData::Io(e))
    }
}

impl From<tera::Error> for RunnerError {
    fn from(e: tera::Error) -> Self {
        RunnerError(
            format!("Template Error: {}", e.to_string()),
            RunnerErrorData::TemplateError(e),
        )
    }
}
