use std::any::Any;
use std::fs;
use std::path::Path;
use kdl::{KdlDocument, KdlEntry, KdlIdentifier, KdlNode};
use serde_yaml::Value;
use crate::tasks::{Task, TaskFile, TaskFileReadError, Variable};



