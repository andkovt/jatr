use std::io::Write;
mod utils;
mod tasks;
mod runner;
mod output;
mod templater;

use crate::runner::ActionRunner;
use crate::tasks::{read_taskfile, Task, TaskFile, TaskFileReadError};
use crate::templater::Templater;
use camino::Utf8Path;
use clap::ArgAction;
use log::{error, Level, LevelFilter};
use std::env::args_os;
use std::{env, io};

const FILE: &str = "tasks.yaml";

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
    builder.format_timestamp(None);

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

fn main() {
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
                    std::process::exit(1);
                },
                _ => {
                    error!("{}", e);
                },
            }

            std::process::exit(1);
        }
        Err(TaskFileReadError::ParseError(e)) => {
            println!("{}", format!("Parse Error: {:?}", e));
            std::process::exit(1);
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

    match run_task(name, task, path.parent().unwrap().as_str(), matches, &file) {
        Ok(exit_code) => {
            if exit_code == 0 {
                output::success("Success");
            }

            std::process::exit(exit_code)
        },
        Err(e) => {
            error!("Unexpected error running task: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn run_task(
    name: &str,
    task: &Task,
    work_dir: &str,
    _args: &clap::ArgMatches,
    tasks: &TaskFile
) -> Result<i32, io::Error> {
    let templater = Templater::for_task(task, tasks);
    let mut runner = ActionRunner::for_task(name, task, work_dir, templater);

    match runner.run(tasks) {
        Ok(_) => {}
        Err(e) => {
            error!("Error running tasks: {:?}", e);
        }
    }

    Ok(0)
}