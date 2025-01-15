use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::str::FromStr;
use void::Void;

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskFile {
    pub variables: Vec<Variable>,
    pub tasks: HashMap<String, Task>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<Argument>,
    pub actions: Vec<Action>,
    pub variables: Vec<Variable>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    Noop,
    Command(ActionCommand),
    If(ActionCommand),
    Task(TaskCall),
}

impl FromStr for Action {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Action::Command(ActionCommand {
            command: String::from(s),
            shell: None,
            tty: true,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Argument {
    pub name: String,
}
impl FromStr for Argument {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Argument {
            name: String::from(s),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionCommand {
    pub command: String,
    pub shell: Option<String>,
    pub tty: bool,
}

impl FromStr for ActionCommand {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ActionCommand {
            command: String::from(s),
            shell: None,
            tty: true,
        })
    }
}

impl Default for ActionCommand {
    fn default() -> Self {
        ActionCommand {
            command: String::new(),
            shell: None,
            tty: true,
        }
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
pub enum VariableValue {
    Static(Value),
    Action(Action),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: VariableValue,
}

impl Default for Task {
    fn default() -> Self {
        Task {
            name: String::new(),
            description: None,
            arguments: vec![],
            actions: vec![],
            variables: vec![],
        }
    }
}

impl Default for TaskFile {
    fn default() -> Self {
        TaskFile {
            variables: vec![],
            tasks: HashMap::new(),
        }
    }
}