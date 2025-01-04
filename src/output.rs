use colored::Colorize;
use log::info;

pub fn cmd_execution(task: &str, cmd: &str) {
    // println!("Task '{}' cmd: {}", task.yellow(), cmd.green());

    action_println(task, "cmd", cmd.green().to_string().as_str());

    // info!("Task '{}' cmd: {}",task, cmd);
}

pub fn if_execution(task: &str, cmd: &str, success: bool, reason: &str) {
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

pub fn if_false(task: &str, cmd: &str, reason: &str) {
    println!("Condition is false, skipping rest of '{task}' tasks. Reason: {reason}");
    // info!("Condition is false, skipping rest of '{task}' tasks. Reason: {reason}");
    // info!("Task '{}' if: {}" ,task, cmd);
}

fn action_println(task: &str, action: &str, content: &str) {
    println!(
        "{}{}{}{}{}{}",
        "Task '".white(),
        task,
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
