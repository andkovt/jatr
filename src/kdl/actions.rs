use crate::kdl::ParseError;
use crate::tasks::{Action, ActionCommand, TaskCall};
use kdl::KdlNode;

pub fn parse_actions(node: &KdlNode, simplified: bool) -> Result<Vec<Action>, ParseError> {
    let mut parsed = vec![];

    for child in node.iter_children() {
        let action_type = child.name().value();

        match action_type {
            "task" => parsed.push(parse_action_task(child)?),
            "if" => parsed.push(Action::If(parse_action_cmd(child)?)),
            "cmd" => parsed.push(Action::Command(parse_action_cmd(child)?)),
            v if !simplified => {
                return Err(ParseError::UnsupportedType(String::from(v)))
            },
            _ => continue,
        }
    }

    Ok(parsed)
}

pub fn parse_action_task(node: &KdlNode) -> Result<Action, ParseError> {
    let Some(entry) = node.entry(0) else {
        return Err(ParseError::MissingArgument(String::from("task")));
    };

    Ok(Action::Task(TaskCall {
        name: String::from(entry.value().as_string().unwrap()),
    }))
}

pub fn parse_action_cmd(node: &KdlNode) -> Result<ActionCommand, ParseError> {
    let Some(entry) = node.entry(0) else {
        return Err(ParseError::MissingArgument(String::from("cmd")));
    };

    let cmd = entry.value().as_string().unwrap();
    let mut shell = None;

    if let Some(shell_entry) = node.entry("shell") {
        shell = match shell_entry.value().as_string() {
            Some(shell) => Some(String::from(shell)),
            None => return Err(ParseError::InvalidArgument(String::from("shell"), shell_entry.to_string())),
        }
    }

    Ok(ActionCommand{
        command: String::from(cmd),
        shell,
    })
}