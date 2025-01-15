use serde::{Deserialize, Serialize};
// use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TaskFile {
    pub variables: Vec<Variable>,
    pub tasks: HashMap<String, Task>,
}

#[derive(Debug, Clone)]
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
    Cd(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Argument {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionCommand {
    pub command: String,
    pub shell: Option<String>,
    pub tty: bool,
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

#[derive(Debug, Clone)]
pub enum VariableValue {
    Static(Value),
    Action(Action),
}

#[derive(Debug, Clone, Serialize)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    List(Vec<Value>),
    Bool(bool),
    Null
}

#[derive(Debug, Clone)]
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