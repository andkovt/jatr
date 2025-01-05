use std::io;
use std::process::ExitStatus;
use serde_yaml::Value;
use tera::{Context, Error, Tera};
use crate::runner::CmdStatus;
use crate::tasks::{Action, ActionCommand, Task, TaskFile, Variable};
use crate::templater::TemplateError::VariableUnexpected;

#[derive(Debug)]
#[allow(unused)]
pub enum TemplateError {
    VariableUnexpected(String),
    VariableResolveIO(io::Error),
    VariableResolveNonZeroExit{
        cmd: String,
        stderr: String,
        status: ExitStatus
    },
    Generic{
        template: String,
        error: Error
    }
}

pub struct Templater {
    context: Context,
    variables: Vec<Variable>

}

impl Templater {
    pub fn for_task(task: &Task, task_file: &TaskFile) -> Self {
        let mut vars = Self::gather_variables(&task_file.variables);
        vars.append(&mut Self::gather_variables(&task.variables));

        Templater {
            context: Context::new(),
            variables: vars
        }
    }

    pub fn template(&self, action: &Action) -> Result<Action, TemplateError> {
        Ok(match action {
            Action::Command(cmd) => {
                Action::Command(self.template_action_command(cmd.clone())?)
            },
            Action::If(cmd) => {
                Action::If(self.template_action_command(cmd.clone())?)
            },
            a => a.clone(),
        })
    }

    pub fn resolve_variables<F: Fn(&str, Option<&String>, &str) -> Result<CmdStatus, TemplateError>>(&mut self, f: F) -> Result<(), TemplateError> {
        for var in &self.variables {
            match (&var.value, &var.cmd) {
                (Some(v), None) => {
                    let res = Self::process_variable_template(&self.context, v.clone())?;
                    self.context.insert(&var.name, &res);
                },
                (None, Some(cmd)) => {
                    let processed = Self::process_variable_template(&self.context, Value::String(String::from(cmd)))?;
                    let processed_cmd = processed.as_str().unwrap();

                    let result = f(var.name.as_str(), var.shell.as_ref(), processed_cmd);

                    match result {
                        Err(e) => return Err(e),
                        Ok(status) => {
                            if !status.exit_status.success() {
                                return Err(TemplateError::VariableResolveNonZeroExit{
                                    cmd: var.name.clone(),
                                    stderr: status.stderr,
                                    status: status.exit_status,
                                })
                            }

                            let stdout = status.stdout.trim();
                            self.context.insert(&var.name, stdout);
                        }
                    }
                },
                (_,_) => return Err(VariableUnexpected(String::from("Unexpected variable value and cmd combination")))
            }
        }

        Ok(())
    }

    fn process_variable_template(context: &Context, value: Value) -> Result<Value, TemplateError> {
        let processed_value = match &value {
            Value::String(s) => {
                let new_val = Tera::one_off(s, &context, false)
                    .map_err(|e| TemplateError::Generic{ template: String::from(s), error: e })?;

                Value::String(new_val)
            },
            val => val.clone(),
        };

        Ok(processed_value)
    }

    fn gather_variables(vars: &Vec<Variable>) -> Vec<Variable> {
        let mut shell_vars = vec![];

        for var in vars {
            shell_vars.push(var.clone());
         }

        shell_vars
    }

    fn template_action_command(
        &self,
        mut cmd: ActionCommand,
    ) -> Result<ActionCommand, TemplateError> {
        cmd.command = Tera::one_off(cmd.command.as_str(), &self.context, false)
            .map_err(|e| TemplateError::Generic {
                template: cmd.command,
                error: e
            })?;

        Ok(cmd)
    }

}