mod reader;
mod runner;
mod tasks;
mod utils;

use crate::tasks::{Task, TaskFile};
use camino::Utf8Path;
use clap::ArgAction;
use colored::Colorize;
use log::{error, LevelFilter};
use runner::environment::RunnerEnvironment;
use runner::{Runner, RunnerResult};
use std::env::args_os;
use std::{env, io};

const FILE: &str = "tasks.kdl";

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
                .action(ArgAction::Set),
        ])
}

fn main() {
    let file = get_file_from_args().unwrap_or_else(|| String::from(FILE));
    let path = Utf8Path::from_path(env::current_dir().unwrap().as_path())
        .unwrap()
        .join(Utf8Path::new(&file));

    setup_logging(get_verbose_from_args());

    let task_file = match reader::open_and_read(&path) {
        Ok(t) => t,
        Err(e) => {
            error!("Error reading file: {}. Error: {:#?}", path, e);
            std::process::exit(1);
        }
    };

    let mut cmd = bootstrap_cmd();
    for (name, task) in &task_file.tasks {
        let about = task.description.clone().unwrap_or_else(String::new);
        let subc = clap::command!(name).about(about);

        cmd = cmd.subcommand(subc);
    }


    let after_help: &'static str = color_print::cstr!(
r#"{usage-heading} {usage}

<bold><underline>Tasks:</underline></bold>
{subcommands}

<bold><underline>Options:</underline></bold>
{options}
{after-help}

"#);

    cmd = cmd.help_template(after_help);

    let global_matches = cmd.get_matches();
    let (name, matches) = match global_matches.subcommand() {
        Some((name, matches)) => (name, matches),
        None => unreachable!("Subcommand not found"),
    };

    let Some(task) = task_file.tasks.get(name) else {
        unreachable!("Task not found")
    };

    match run_task(
        task,
        path.parent().unwrap().as_str(),
        matches,
        &task_file,
    ) {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(e) => {
            error!("Unexpected error running task: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn run_task<'a>(
    task: &'a Task,
    work_dir: &str,
    _args: &clap::ArgMatches,
    tasks: &'a TaskFile,
) -> Result<i32, io::Error> {
    let mut env = RunnerEnvironment::default();
    env.work_dir(work_dir).unwrap();

    let mut runner = Runner::for_taskfile(tasks, env);

    match runner.run(task) {
        Ok(RunnerResult::Success) => {
            let s = "Success".green();
            println!("{}", s);
            return Ok(0);
        }
        Ok(RunnerResult::Skipped) => {
            let s = "Skipped".yellow();
            println!("{}", s);
            return Ok(0);
        }
        Ok(RunnerResult::Failure) => {
            let s = "Failure".red();
            println!("{}", s);
            return Ok(1);
        }
        Err(e) => {
            error!("Error running tasks: {:?}", e);
        }
    }

    Ok(0)
}
