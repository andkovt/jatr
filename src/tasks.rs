use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct TaskFile {
    pub variables: Vec<Variable>,
    pub tasks: HashMap<String, Task>,
}

#[derive(Debug, Clone, Default)]
pub struct Task {
    pub name: String,
    pub description: Option<String>,
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ActionCommand {
    pub command: String,
    pub shell: Option<String>,
    pub tty: bool,
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