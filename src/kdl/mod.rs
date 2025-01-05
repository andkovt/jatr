use std::fs;
use std::path::Path;
use camino::Utf8Path;
use kdl::{KdlDocument, KdlError};
use log::error;
use crate::tasks::TaskFile;

mod variables;
mod tasks;
mod actions;

#[derive(Debug)]
pub enum ParseError {
    Other,
    SyntaxError(KdlError),
    // block "arg1" "arg2" <-- Arguments
    // block "arg1" prop1=true
    UnknownProp(String),
    UnknownPropValue(String, String),
    UnsupportedArrayKey(String),
    UnsupportedArrayValue(String, serde_yaml::Error),
    UnsupportedType(String),

    MissingValue(String),
    MissingBody(String),
    MissingArgument(String),

    InvalidArgument(String, String)
}

pub fn parse(content: String) -> Result<TaskFile, ParseError> {
    let doc: KdlDocument = match content.parse() {
        Ok(doc) => doc,
        Err(e) => {
            return Err(ParseError::SyntaxError(e))
        },
    };

    let mut file = TaskFile {
        variables: vec![],
        tasks: Default::default(),
    };

    let vars = doc.get("variables");
    let tasks = doc.get("tasks");

    if let Some(node) = vars {
        match variables::parse_variables(node) {
            Ok(vars) => {
                file.variables = vars;
            }
            Err(e) => {
                error!("Error parsing variables section: {:?}", e)
            }
        }
    }

    if let Some(node) = tasks {
        match tasks::parse_tasks(node) {
            Ok(tasks) => {
                file.tasks = tasks;
            }
            Err(e) => {
                error!("Error parsing variables section: {:?}", e)
            }
        }
    }

    Ok(file)
}