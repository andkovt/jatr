use tera::{Context, Error, Tera};
use crate::tasks::{Action, ActionCommand, Task, TaskFile, Variable};


#[derive(Debug)]
pub enum TemplateError {
    Generic{
        template: String,
        error: Error
    }
}

pub struct Templater {
    context: Context,
}

impl Templater {
    pub fn for_task(task: &Task, task_file: &TaskFile) -> Result<Self, TemplateError> {
        let mut context = Context::new();

        context = Self::load_variables(context, &task_file.variables)?;
        context = Self::load_variables(context, &task.variables)?;

        Ok(Templater {
            context
        })
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

    fn load_variables(mut context: Context, vars: &Vec<Variable>) -> Result<Context, TemplateError> {
        for var in vars {
            let processed_value = match &var.value {
                serde_yaml::Value::String(s) => {
                    let new_val = Tera::one_off(s, &context, false)
                        .map_err(|e| TemplateError::Generic{ template: String::from(s), error: e })?;

                    serde_yaml::Value::String(new_val)
                }
                val => val.clone(),
            };

            context.insert(&var.name, &processed_value);
        }

        Ok(context)
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