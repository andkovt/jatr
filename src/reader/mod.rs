mod node;
mod parser;

use camino::Utf8Path;
use kdl::KdlDocument;
use std::{fs, io};

use crate::tasks::TaskFile;

#[derive(Debug)]
#[allow(dead_code)]
pub enum TaskFileReadError {
    Io(io::Error),
    Syntax(kdl::KdlError),
    Parser(parser::ParserError),
}

pub fn open_and_read(path: &Utf8Path) -> Result<TaskFile, TaskFileReadError> {
    let content = fs::read_to_string(path)?;

    read(content)
}

fn read(content: String) -> Result<TaskFile, TaskFileReadError> {
    let doc: KdlDocument = content.parse()?;
    let mut task_file = TaskFile::default();
    let mut context = parser::Context::default();

    for node in doc.nodes() {
        parser::parse_node(node, &mut task_file, &mut context)?;
    }

    Ok(task_file)
}

impl From<io::Error> for TaskFileReadError {
    fn from(e: io::Error) -> Self {
        TaskFileReadError::Io(e)
    }
}

impl From<kdl::KdlError> for TaskFileReadError {
    fn from(e: kdl::KdlError) -> Self {
        TaskFileReadError::Syntax(e)
    }
}

impl From<parser::ParserError> for TaskFileReadError {
    fn from(e: parser::ParserError) -> Self {
        TaskFileReadError::Parser(e)
    }
}
