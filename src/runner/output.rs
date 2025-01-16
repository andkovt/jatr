use color_print::{cformat, cstr};

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
        action_println(
            &self.task_name,
            "cmd",
            cformat!("<green>'{}'</green>", cmd).as_str(),
        );
    }

    pub fn cd_execution(&self, cd: &str) {
        action_println(
            &self.task_name,
            "cd",
            cformat!("<green>'{}'</green>", cd).as_str(),
        );
    }

    pub fn if_execution(&self, cmd: &str, success: bool) {
        let res_str = match success {
            true => cstr!("<green>true</green>"),
            false => cstr!("<yellow>false</yellow>"),
        };

        action_println(
            &self.task_name,
            "if",
            cformat!(
                "<bright-black>'{}'</><white> == '</>{}<bright-black>'</>",
                cmd,
                res_str
            )
            .as_str(),
        );
    }
}

fn action_println(task: &str, action: &str, content: &str) {
    println!(
        "{}",
        cformat!(
            "<white>Task '<yellow>{}</>' {}:</> {}",
            task,
            action,
            content
        )
    );
}
