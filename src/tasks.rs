use std::collections::HashMap;
use std::{fs, io};
use std::str::FromStr;
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use serde_nested_with::serde_nested;
use serde_yaml::Value;
use void::Void;

#[derive(Debug)]
pub enum TaskFileReadError {
    IOError(io::Error),
    ParseError(serde_yaml::Error),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskFile {
    #[serde(alias = "vars", default = "default_variables")]
    pub variables: Vec<Variable>,
    pub tasks: HashMap<String, Task>,
}

#[serde_nested]
#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    #[serde(rename = "desc")]
    pub description: Option<String>,

    #[serde(rename = "if", default = "default_conditions")]
    #[serde_nested(sub = "ActionCommand", serde(deserialize_with = "crate::utils::string_or_struct"))]
    pub conditions: Vec<ActionCommand>,

    #[serde(rename = "args", default = "default_arguments")]
    #[serde_nested(sub = "Argument", serde(deserialize_with = "crate::utils::string_or_struct"))]
    pub arguments: Vec<Argument>,

    #[serde(rename = "actions")]
    #[serde_nested(sub = "Action", serde(deserialize_with = "crate::utils::string_or_struct"))]
    pub actions: Vec<Action>,

    #[serde(alias = "vars", default = "default_variables")]
    pub variables: Vec<Variable>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    #[serde(alias = "noop")]
    Noop,
    #[serde(alias = "cmd", deserialize_with = "crate::utils::string_or_struct")]
    Command(ActionCommand),
    #[serde(alias = "if", deserialize_with = "crate::utils::string_or_struct")]
    If(ActionCommand),
    #[serde(alias = "task", deserialize_with = "crate::utils::string_or_struct")]
    Task(TaskCall),
}
impl FromStr for Action {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Action::Command(ActionCommand {
            command: String::from(s),
            shell: None,
            parallel: Some(false),
        }))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Argument {
    pub name: String
}
impl FromStr for Argument {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Argument{
            name: String::from(s),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionCommand {
    #[serde(rename = "cmd")]
    pub command: String,
    pub shell: Option<String>,
    pub parallel: Option<bool>
}
impl FromStr for ActionCommand {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ActionCommand {
            command: String::from(s),
            shell: None,
            parallel: Some(false),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskCall {
    pub name: String,
}
impl FromStr for TaskCall {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TaskCall {
            name: String::from(s),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variable {
    pub name: String,
    #[serde(alias = "val")]
    pub value: Option<Value>,
    pub cmd: Option<String>,
    pub shell: Option<String>,
}


pub fn read_taskfile(file_path: &Utf8Path) -> Result<TaskFile, TaskFileReadError> {
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => return Err(TaskFileReadError::IOError(e)),
    };

    let task_file: TaskFile = match serde_yaml::from_str(content.as_str()) {
        Ok(file) => file,
        Err(e) => return Err(TaskFileReadError::ParseError(e)),
    };

    Ok(task_file)
}


fn default_conditions() -> Vec<ActionCommand> {
    vec![]
}

fn default_arguments() -> Vec<Argument> {
    vec![]
}

fn default_variables() -> Vec<Variable> {
    vec![]
}