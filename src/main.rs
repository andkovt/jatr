use std::io::Write;
mod utils;
mod tasks;
mod runner;
mod output;
mod templater;

use crate::runner::{ActionRunner, RunActionError, RunResult};
use crate::tasks::{read_taskfile, ActionCommand, Task, TaskFile, TaskFileReadError, Variable};
use camino::Utf8Path;
use clap::error::ErrorKind;
use clap::ArgAction;
use log::{debug, error, info, log_enabled, Level, LevelFilter};
use serde::{Deserialize, Deserializer, Serialize};
use serde_nested_with::serde_nested;
use std::collections::HashMap;
use std::env::args_os;
use std::path::Path;
use std::process::{ExitCode, ExitStatus};
use std::str::FromStr;
use std::{env, error::Error, fs, io, process};
use tera::{Context, Tera};
use void::{unreachable, Void};
use crate::templater::Templater;

const FILE: &str = "tasks.yaml";

struct CommandStatus {
    exit_code: ExitStatus,
}

fn get_file_from_args() -> Option<String> {
    let mut is_file = false;
    for arg in args_os() {
        if is_file {
            return Some(String::from(arg.to_str().unwrap()));
        }

        if arg == "-f" || arg == "--file" {
            is_file = true;
        }
    }

    None
}

fn get_verbose_from_args() -> bool {
    args_os().any(|x| &x == "-v" || &x == "--verbose")
}

fn setup_logging(verbose: bool) {
    let mut builder = env_logger::builder();

    builder.format(|buf, record| {
        let level_style = buf.default_level_style(record.level());
        match record.level() {
            Level::Info => {
                writeln!(buf, "{level_style}{}{level_style:#}", record.args())
            }
            _ => {
                writeln!(buf, "{level_style}{}{level_style:#} {}", record.level(), record.args())
            }
        }
    });

    builder.filter_level(match verbose {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    });

    builder.init();
}

fn bootstrap_cmd() -> clap::Command {
    clap::Command::new("jatr")
        .bin_name("jatr")
        .arg_required_else_help(true)
        .subcommand_required(true)
        .args(vec![
            clap::arg!(verbose: -v --verbose "Enables verbose output")
                .global(true)
                .action(ArgAction::SetTrue),
            clap::arg!(file: -f --file "Specify task file")
                .global(true)
                .action(ArgAction::Set)
        ])
}

fn play_tmpl(vars: &Vec<Variable>) {
    let mut tera = Tera::default();
    tera.add_raw_template("test", "Hello {{ name }}");

    let mut context = Context::new();
    context.insert("name", "andrew");
    context.insert("rr", &vec!["one", "two"]);

    for var in vars {
        context.insert(&var.name, &var.value);
    }

    let result = Tera::one_off(
        "Poop: {{ poop }}, oth: {{other | join(sep='.')}}, Hello {{ name }} {{ rr | join(sep=\",\") }}",
        &context,
        true
    );

    println!("{}", result.unwrap());

}

fn main() {
    // let mut tera = Tera::default();
    // tera.add_raw_template("test", "Hello {{ name }}");
    //
    // let mut context = Context::new();
    // context.insert("name", "andrew");
    // context.insert("rr", &vec!["one", "two"]);
    //
    // let result = Tera::one_off("Hello {{ name }} {{ rr | join(sep=\",\") }}", &context, true);
    //
    // println!("{}", result.unwrap());
    //
    // return;

    let file = get_file_from_args().unwrap_or_else(|| String::from(FILE));
    let path = Utf8Path::from_path(env::current_dir().unwrap().as_path()).unwrap().join(Utf8Path::new(&file));
    setup_logging(get_verbose_from_args());

    let mut cmd = bootstrap_cmd();

    let file = match read_taskfile(&path) {
        Ok(file) => file,
        Err(TaskFileReadError::IOError(e)) => {
            match e.kind() {
                io::ErrorKind::NotFound => {
                    error!("Tasks file not found at {}", path);
                    return;
                },
                _ => {
                    error!("{}", e);
                },
            }

            return;
        }
        Err(TaskFileReadError::ParseError(e)) => {
            println!("{}", format!("Parse Error: {:?}", e));
            return;
        }
    };


    for (name, task) in &file.tasks {
        let about = task.description.clone().unwrap_or_else(String::new);
        let subc = clap::command!(name).about(about);

        cmd = cmd.subcommand(subc);
    }

    let global_matches = cmd.get_matches();
    let (name, matches) = match global_matches.subcommand() {
        Some((name, matches)) => (name, matches),
        None => unreachable!("Subcommand not found")
    };

    let Some(task) = file.tasks.get(name) else {
      unreachable!("Task not found")
    };

    // println!("{:#?}", file);
    // play_tmpl(&file.tasks.get("up").unwrap().variables);
    //
    // return;

    run_task(name, task, path.parent().unwrap().as_str(), matches, &file);
}

fn run_task(
    name: &str,
    task: &Task,
    work_dir: &str,
    args: &clap::ArgMatches,
    tasks: &TaskFile
) -> Result<(), io::Error> {
    let templater = Templater::for_task(task, tasks);
    let runner = ActionRunner::for_task(name, task, work_dir, templater.unwrap());
    match runner.run(tasks) {
        Ok(_) => {}
        Err(e) => {
            error!("Error running tasks: {:?}", e);
        }
    }

    Ok(())
}