use super::{node, open_and_read};
use crate::tasks::{ActionCommand, Task, TaskCall, Value};
use crate::{
    tasks::{Action, TaskFile, Variable, VariableValue},
    utils::kdl_value_to_value,
    S,
};
use camino::Utf8Path;
use kdl::KdlNode;

#[derive(Debug, Clone)]
pub enum Scope {
    Global,
    Variables,
    Variable,
    Tasks,
    Task,
    Actions,
}

#[derive(Debug)]
pub struct ContextError(String);

#[derive(Debug)]
#[allow(dead_code)]
pub struct ParserError(String, ParserErrorData);

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParserErrorData {
    UnknownNode { name: String },
    MissingArgument { name: String },
    MissingBody,

    ImportError,
    ContextError(ContextError),
    InvalidType,
}

#[derive(Debug)]
pub struct ContextScope {
    pub scope: Scope,
    actions: Vec<Action>,
    variables: Vec<Variable>,
    list_items: Vec<Value>,
    task: Task,
}

#[derive(Debug)]
pub struct Context {
    scopes: Vec<ContextScope>,
}

pub fn parse_node(
    node: &KdlNode,
    task_file: &mut TaskFile,
    context: &mut Context,
) -> Result<(), ParserError> {
    let node_name = node.name().value();
    let Some(node_type) = node::get_node_type_by_name(node_name, context) else {
        return Err(ParserError(
            format!("Unknown node '{node_name}'"),
            ParserErrorData::UnknownNode {
                name: node_name.to_string(),
            },
        ));
    };

    match node_type {
        node::NodeType::Import => parse_import_node(node, task_file, context),
        node::NodeType::Variables => parse_variables_node(node, task_file, context),
        node::NodeType::Variable => parse_variable_node(node, task_file, context),
        node::NodeType::ListItem => parse_list_item(node, task_file, context),
        node::NodeType::Cmd => parse_cmd(node, task_file, context),
        node::NodeType::Tasks => parse_tasks(node, task_file, context),
        node::NodeType::Task => parse_task(node, task_file, context),
        node::NodeType::Actions => parse_actions(node, task_file, context),
        node::NodeType::If => parse_if(node, task_file, context),
        node::NodeType::ActionTask => parse_action_task(node, task_file, context),
        node::NodeType::ActionCd => parse_action_cd(node, task_file, context),
    }
}

pub fn parse_import_node(
    node: &KdlNode,
    task_file: &mut TaskFile,
    _context: &mut Context,
) -> Result<(), ParserError> {
    let Some(file_path) = node.get(0) else {
        return Err(ParserError(
            S!("Missing argument 'file'"),
            ParserErrorData::MissingArgument { name: S!("file") },
        ));
    };

    let prefix = match node.get("prefix") {
        Some(prefix_value) => prefix_value.as_string(),
        None => None,
    };

    let path = Utf8Path::new(file_path.as_string().unwrap());
    let imported_file = match open_and_read(path) {
        Ok(file) => file,
        Err(e) => {
            return Err(ParserError(
                format!("Error reading file {:?}", e),
                ParserErrorData::ImportError,
            ))
        }
    };

    task_file.variables.extend(imported_file.variables);
    for (name, task) in imported_file.tasks {
        task_file
            .tasks
            .insert(format!("{}:{}", prefix.unwrap_or(""), name), task);
    }

    Ok(())
}

pub fn parse_variables_node(
    node: &KdlNode,
    task_file: &mut TaskFile,
    context: &mut Context,
) -> Result<(), ParserError> {
    context.push_scope(Scope::Variables);

    if let Some(children) = node.children() {
        for child in children.nodes() {
            parse_node(child, task_file, context)?;
        }
    }

    let scoped_context = context.pop_scope();

    match &context.current_scope().scope {
        Scope::Global => task_file.variables.extend(scoped_context.variables),
        Scope::Task => context
            .current_scope()
            .task
            .variables
            .extend(scoped_context.variables),
        scope => {
            return Err(ParserError(
                format!("Cannot add variables to scope '{:?}'", scope),
                ParserErrorData::ContextError(ContextError(format!(
                    "Cannot add variables to scope '{:?}'",
                    scope
                ))),
            ))
        }
    }

    Ok(())
}

pub fn parse_variable_node(
    node: &KdlNode,
    task_file: &mut TaskFile,
    context: &mut Context,
) -> Result<(), ParserError> {
    let name = node.name().value();
    let Some(value) = node.get(0) else {
        return parse_variable_node_body(node, task_file, context);
    };

    context.add_variable(Variable {
        name: name.to_string(),
        value: VariableValue::Static(kdl_value_to_value(value)),
    });

    Ok(())
}

pub fn parse_variable_node_body(
    node: &KdlNode,
    task_file: &mut TaskFile,
    context: &mut Context,
) -> Result<(), ParserError> {
    let Some(children) = node.children() else {
        return Err(ParserError(
            S!("Vairbale should not be empty"),
            ParserErrorData::MissingBody,
        ));
    };

    context.scope(Scope::Variable);
    for child in children.nodes() {
        parse_node(child, task_file, context)?;
    }

    let scope_context = context.pop_scope();
    let mut actions = scope_context.actions;
    let list_items = scope_context.list_items;

    if !actions.is_empty() && !list_items.is_empty() {
        return Err(ParserError(
            S!("Cannot mix actions and list items inside variable"),
            ParserErrorData::ContextError(ContextError(S!("Cannot mix actions and list items"))),
        ));
    }

    if !list_items.is_empty() {
        context.add_variable(Variable {
            name: node.name().value().to_string(),
            value: VariableValue::Static(Value::List(list_items)),
        });
    }

    if !actions.is_empty() && actions.len() > 1 {
        return Err(ParserError(
            S!("Cannot have more than one action"),
            ParserErrorData::ContextError(ContextError(S!("Cannot have more than one action"))),
        ));
    } else if !actions.is_empty() {
        context.add_variable(Variable {
            name: node.name().value().to_string(),
            value: VariableValue::Action(actions.remove(0)),
        });
    }

    Ok(())
}

pub fn parse_list_item(
    node: &KdlNode,
    _task_file: &mut TaskFile,
    context: &mut Context,
) -> Result<(), ParserError> {
    match node.get(0) {
        None => context.add_list_item(Value::Null),
        Some(value) => context.add_list_item(kdl_value_to_value(value)),
    }

    Ok(())
}

pub fn parse_tasks(
    node: &KdlNode,
    task_file: &mut TaskFile,
    context: &mut Context,
) -> Result<(), ParserError> {
    context.scope(Scope::Tasks);

    if let Some(children) = node.children() {
        for child in children.nodes() {
            parse_node(child, task_file, context)?;
        }
    }

    context.pop_scope();

    Ok(())
}

pub fn parse_actions(
    node: &KdlNode,
    task_file: &mut TaskFile,
    context: &mut Context,
) -> Result<(), ParserError> {
    context.scope(Scope::Actions);

    if let Some(children) = node.children() {
        for child in children.nodes() {
            parse_node(child, task_file, context)?;
        }
    }

    let scoped_context = context.pop_scope();

    // Move found actions to task scope
    if matches!(context.current_scope().scope, Scope::Task) {
        for action in scoped_context.actions {
            context.add_action(action);
        }
    }

    Ok(())
}

pub fn parse_task(
    node: &KdlNode,
    task_file: &mut TaskFile,
    context: &mut Context,
) -> Result<(), ParserError> {
    let task_name = node.name().value();
    let description_value = match node.get(0) {
        Some(arg) => arg.as_string(),
        None => None,
    };

    let task = Task {
        name: String::from(task_name),
        description: description_value.map(|x| x.to_string()),
        ..Default::default()
    };

    context.task_scope(task);

    if let Some(children) = node.children() {
        for child in children.nodes() {
            parse_node(child, task_file, context)?;
        }
    }

    let mut scoped_context = context.pop_scope();
    scoped_context.task.actions.extend(scoped_context.actions);

    task_file.tasks.insert(
        String::from(scoped_context.task.name.clone()),
        scoped_context.task,
    );

    Ok(())
}

pub fn parse_cmd(
    node: &KdlNode,
    _task_file: &mut TaskFile,
    context: &mut Context,
) -> Result<(), ParserError> {
    let Some(command) = node.get(0) else {
        return Err(ParserError(
            S!("Command is missing"),
            ParserErrorData::MissingArgument {
                name: S!("command"),
            },
        ));
    };

    let Some(command) = command.as_string() else {
        return Err(ParserError(
            format!("Command should be a string, got {:?}", command),
            ParserErrorData::InvalidType,
        ));
    };

    let shell = match node.get("shell") {
        None => None,
        Some(prop) => match prop.as_string() {
            None => {
                return Err(ParserError(
                    format!("Shell should be a string, got {:?}", prop),
                    ParserErrorData::InvalidType,
                ))
            }
            Some(shell) => Some(String::from(shell)),
        },
    };

    let tty = match context.current_scope().scope {
        Scope::Variable => false,
        _ => true,
    };

    let cmd = ActionCommand {
        command: command.to_string(),
        shell,
        tty,
    };

    context.add_action(Action::Command(cmd));

    Ok(())
}

pub fn parse_if(
    node: &KdlNode,
    _task_file: &mut TaskFile,
    context: &mut Context,
) -> Result<(), ParserError> {
    let Some(command) = node.get(0) else {
        return Err(ParserError(
            S!("Command is missing"),
            ParserErrorData::MissingArgument {
                name: S!("command"),
            },
        ));
    };

    let Some(command) = command.as_string() else {
        return Err(ParserError(
            format!("Command should be a string, got {:?}", command),
            ParserErrorData::InvalidType,
        ));
    };

    let shell = match node.get("shell") {
        None => None,
        Some(prop) => match prop.as_string() {
            None => {
                return Err(ParserError(
                    format!("Shell should be a string, got {:?}", prop),
                    ParserErrorData::InvalidType,
                ))
            }
            Some(shell) => Some(String::from(shell)),
        },
    };

    let cmd = ActionCommand {
        command: command.to_string(),
        shell,
        tty: false,
    };

    context.add_action(Action::If(cmd));

    Ok(())
}

pub fn parse_action_task(
    node: &KdlNode,
    _task_file: &mut TaskFile,
    context: &mut Context,
) -> Result<(), ParserError> {
    let Some(task) = node.get(0) else {
        return Err(ParserError(
            S!("Task is missing"),
            ParserErrorData::MissingArgument { name: S!("task") },
        ));
    };

    let Some(task) = task.as_string() else {
        return Err(ParserError(
            format!("Task name should be a string, got {:?}", task),
            ParserErrorData::InvalidType,
        ));
    };

    context.add_action(Action::Task(TaskCall {
        name: String::from(task),
    }));

    Ok(())
}

pub fn parse_action_cd(
    node: &KdlNode,
    _task_file: &mut TaskFile,
    context: &mut Context,
) -> Result<(), ParserError> {
    let Some(path) = node.get(0) else {
        return Err(ParserError(
            S!("Missing argument 'path'"),
            ParserErrorData::MissingArgument { name: S!("path") },
        ));
    };

    context.add_action(Action::Cd(path.to_string()));

    Ok(())
}

impl Default for Context {
    fn default() -> Self {
        Context {
            scopes: vec![],
        }
    }
}

impl Default for ContextScope {
    fn default() -> Self {
        ContextScope {
            scope: Scope::Global,
            actions: vec![],
            list_items: vec![],
            variables: vec![],
            task: Task::default(),
        }
    }
}

impl Context {
    pub fn push_scope(&mut self, scope: Scope) {
        self.scope(scope)
    }

    pub fn scope(&mut self, scope: Scope) {
        self.scopes.push(ContextScope {
            scope,
            ..Default::default()
        });
    }

    pub fn task_scope(&mut self, task: Task) {
        self.scope(Scope::Task);
        self.current_scope().task = task;
    }

    pub fn pop_scope(&mut self) -> ContextScope {
        let scope = self.scopes.pop();
        scope.unwrap_or_else(|| ContextScope::default())
    }

    pub fn current_scope_type(&self) -> Scope {
        self.scopes
            .last()
            .unwrap_or(&ContextScope::default())
            .scope
            .clone()
    }

    pub fn current_scope(&mut self) -> &mut ContextScope {
        if self.scopes.is_empty() {
            self.scopes.push(ContextScope::default());
        }

        self.scopes.last_mut().unwrap()
    }

    pub fn add_variable(&mut self, variable: Variable) {
        self.current_scope().variables.push(variable);
    }

    pub fn add_action(&mut self, action: Action) {
        self.current_scope().actions.push(action);
    }

    pub fn add_list_item(&mut self, item: Value) {
        self.current_scope().list_items.push(item);
    }
}

impl From<ContextError> for ParserError {
    fn from(e: ContextError) -> Self {
        ParserError(
            format!("Context error: {}", e.0),
            ParserErrorData::ContextError(e),
        )
    }
}
