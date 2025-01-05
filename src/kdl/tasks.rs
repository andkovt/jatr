use std::collections::HashMap;
use kdl::{KdlDocument, KdlEntry, KdlNode};
use crate::kdl::{actions, variables, ParseError};
use crate::tasks::Task;

pub fn parse_tasks(node: &KdlNode) -> Result<HashMap<String, Task>, ParseError> {
    let mut parsed: HashMap<String, Task> = HashMap::new();

    for child in node.iter_children() {
        let name = child.name();

        let mut parsed_task = Task {
            name: String::from(name.value()),
            description: None,
            arguments: vec![],
            actions: vec![],
            variables: vec![],
        };

        parsed_task = parse_arguments(parsed_task, child.entries())?;

        let Some(children) = child.children() else {
            return Err(ParseError::MissingBody(String::from(format!("Task {} is missing a body", name))));
        };

        parsed_task = parse_body(parsed_task, children)?;

        // If no action defined, parse the actions from the task node
        if parsed_task.actions.is_empty() && children.get("actions").is_none() {
            parsed_task.actions = actions::parse_actions(child, true)?
        }
        parsed.insert(String::from(name.value()), parsed_task);
    }

    Ok(parsed)
}

fn parse_arguments(mut task: Task, entries: &[KdlEntry]) -> Result<Task, ParseError> {
    let mut desc = None;

    for entry in entries {
        if entry.name().is_some() { // Arguments don't have names
            continue;
        }

        desc = Some(String::from(entry.value().as_string().unwrap()));
    }

    task.description = desc;
    Ok(task)
}
fn parse_body(mut task: Task, children: &KdlDocument) -> Result<Task, ParseError> {
    let variables = children.get("variables");
    let actions = children.get("actions");


    if let Some(variables) = variables {
        task = parse_body_variables(task, variables)?
    }

    if let Some(actions) = actions {
        task = parse_body_actions(task, actions)?
    }

    Ok(task)
}

fn parse_body_variables(mut task: Task, node: &KdlNode) -> Result<Task, ParseError> {
    task.variables = variables::parse_variables(node)?;

    Ok(task)
}

fn parse_body_actions(mut task: Task, node: &KdlNode) -> Result<Task, ParseError> {
    task.actions = actions::parse_actions(node, false)?;

    Ok(task)
}