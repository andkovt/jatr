use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use serde_yaml::Value;
use crate::kdl::ParseError;
use crate::tasks::Variable;

pub fn parse_variables(node: &KdlNode) -> Result<Vec<Variable>, ParseError> {
    let mut parsed = vec![];

    for child in node.iter_children() {
        let name = child.name();
        let mut parsed_var = Variable {
            name: name.to_string(),
            shell: None,
            value: None,
            cmd: None,
        };

        parsed_var = parse_variable_arguments(parsed_var, child.entries())?;

        if let Some(children) = child.children() {
            parsed_var = parse_variable_children(parsed_var, children)?
        }

        parsed.push(parsed_var);
    }

    Ok(parsed)
}

pub fn parse_variable_arguments(mut var: Variable, entries: &[KdlEntry]) -> Result<Variable, ParseError> {
    let mut value = None;
    let mut is_cmd = false;

    for entry in entries {
        let identifier = entry.name();
        let entry_value_str = match entry.value() {
            KdlValue::String(s) => s,
            other => &other.to_string(),
        };

        let Some(id) = identifier else {
            value = Some(String::from(entry_value_str));
            continue;
        };

        match id.value() {
            "type" => {
                match entry_value_str.as_str() {
                    "cmd" => {
                        is_cmd = true;
                    }
                    &_ => return Err(ParseError::UnknownPropValue(String::from(id.value()), String::from(entry_value_str)))
                }
            },
            "shell" => {
                var.shell = Some(String::from(entry_value_str))
            }
            &_ => return Err(ParseError::UnknownProp(String::from(id.value())))
        }
    }

    match is_cmd {
        true => {
            var.cmd = value.clone();
        },
        false => {
            var.value = value.clone().map(Value::String);
        }
    }

    Ok(var)
}

pub fn parse_variable_children(mut var: Variable, children: &KdlDocument) -> Result<Variable, ParseError> {
    let mut values: Vec<Value> = vec![];
    for node in children.nodes() {
        if node.name().value() != "-" {
            return Err(ParseError::UnsupportedArrayKey(String::from(node.name().value())))
        }

        match node.entry(0) {
            Some(entry) => {
                let v = entry.value();
                let value_as_string = v.to_string();

                values.push(match serde_yaml::from_str(value_as_string.as_str()) {
                    Ok(v) => v,
                    Err(e) => return Err(ParseError::UnsupportedArrayValue(value_as_string, e))
                });
            },
            None => return Err(ParseError::MissingValue(node.to_string()))
        }
    }

    if !values.is_empty() {
        var.value = Some(Value::Sequence(values));
    }

    Ok(var)
}