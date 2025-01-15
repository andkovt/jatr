use colored::Colorize;

pub struct Output {
    task_name: String,
}

impl Output {
    pub fn for_task(task_name: &str) -> Self {
        Self {
            task_name: String::from(task_name),
        }
    }
}

impl Output {
    pub fn cmd_execution(&self, cmd: &str) {
        action_println(&self.task_name, "cmd", cmd.green().to_string().as_str());
    }

    pub fn if_execution(&self, cmd: &str, success: bool) {
        action_println(
            &self.task_name,
            "if",
            &format!(
                "{}{}{}{}{}",
                "'".bright_black(),
                cmd.bright_black().to_string().as_str(),
                "' is '".bright_black(),
                match success {
                    true => "true".green(),
                    false => "false".yellow(),
                },
                "'".bright_black(),
            ),
        );
    }
}

fn action_println(task: &str, action: &str, content: &str) {
    println!(
        "{}{}{}{}{}{}{}{}",
        "Task '".white(),
        task.yellow(),
        "' ".white(),
        match action {
            "cmd" => "cmd".white(),
            "if" => "if".white(),
            s => s.yellow(),
        },
        ": ".white(),
        "'".white(),
        content,
        "'".white()
    );
}
