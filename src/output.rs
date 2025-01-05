use colored::Colorize;

pub fn success(msg: &str) {
    println!("{}", msg.green());
}

pub fn cmd_execution(task: &str, cmd: &str) {
    action_println(task, "cmd", cmd.green().to_string().as_str());
}

pub fn if_execution(task: &str, cmd: &str, success: bool) {
    action_println(
        task,
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
        )
    );
}

fn action_println(task: &str, action: &str, content: &str) {
    println!(
        "{}{}{}{}{}{}",
        "Task '".white(),
        task.yellow(),
        "' ".white(),
        match action {
            "cmd" => "cmd".white(),
            "if" => "if".white(),
            s => s.yellow(),
        },
        ": ".white(),
        content
    );
}
