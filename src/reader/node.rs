use super::parser::{self, Scope};

pub enum NodeType {
    Import,
    Variables,
    Variable,
    ListItem,
    Tasks,
    Task,
    Actions,
    Cmd,
    If,
    ActionTask,
}

pub fn get_node_type_by_name(name: &str, context: &parser::Context) -> Option<NodeType> {
    match name {
        "import" => Some(NodeType::Import),
        "variables" => Some(NodeType::Variables),
        "cmd" => Some(NodeType::Cmd),
        "-" => Some(NodeType::ListItem),
        "tasks" => Some(NodeType::Tasks),
        "actions" => Some(NodeType::Actions),
        "if" => Some(NodeType::If),
        "task" => Some(NodeType::ActionTask),
        _ => match context.current_scope_type() {
            Scope::Tasks => Some(NodeType::Task),
            Scope::Variables => Some(NodeType::Variable),
            _ => None,
        },
    }
}
