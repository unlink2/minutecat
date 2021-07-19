use super::logset::LogSet;
use super::logfile::Logfile;
use super::task::Task;
use super::task::ClockTimeSource;
use super::source::FileDataSource;
use super::trigger::{RegexTrigger, TriggerType};
use std::path::PathBuf;
use std::str::FromStr;
use super::dirs;
use super::error::BoxResult;
use super::clap::{AppSettings, Clap};
use std::env;

/**
 * This file descibes a general purpse command line interface
 * that all frontends should implement
 * it provides a way to provide a logset, modify it and list it in simple terms
 *
 * Take note that as opposed to
 * other code in the lib/ module
 * most of the functions here will panic
 * if file system access fails for some reason
 */

pub struct Interface {
    pub logset: LogSet,
    pub options: Opts,
    pub cfg_path: String
}

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Lukas Krickl <lukas@krickl.dev>")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(subcommand)]
    subcmd: Option<SubCommand>
 }

#[derive(Clap)]
pub enum SubCommand {
    #[clap(version = "0.1.0", author = "Lukas Krickl <lukas@krickl.dev>")]
    Add(Add),

    #[clap(version = "0.1.0", author = "Lukas Krickl <lukas@krickl.dev>")]
    List(List),

    #[clap(version = "0.1.0", author = "Lukas Krickl <lukas@krickl.dev>")]
    Delete(Delete),

    #[clap(version = "0.1.0", author = "Lukas Krickl <lukas@krickl.dev>")]
    AddReTrigger(AddReTrigger),

    #[clap(version = "0.1.0", author = "Lukas Krickl <lukas@krickl.dev>")]
    ListTrigger(ListTrigger),

    #[clap(version = "0.1.0", author = "Lukas Krickl <lukas@krickl.dev>")]
    DeleteTrigger(DeleteTrigger)
}

#[derive(Clap)]
pub struct Add {
    name: String,
    location: String,
    line_limit: usize,
    logtype: String,
    refresh_time: String
}

#[derive(Clap)]
pub struct List;

#[derive(Clap)]
pub struct Delete {
    pub index: usize
}

#[derive(Clap)]
pub struct AddReTrigger {
    pub index: usize,
    pub name: String,
    pub desc: String,
    pub trigger_type: String,
    pub regex: String
}

#[derive(Clap)]
pub struct ListTrigger {
    pub index: usize
}

#[derive(Clap)]
pub struct DeleteTrigger {
    pub log_index: usize,
    pub trigger_index: usize
}

// TODO allow user to move config path?
pub fn config_path() -> PathBuf {
    let default = dirs::home_dir().expect("Unable to get home directory!")
            .join(".config/minutecat");
    match env::var("MINUTECATDIR") {
        Ok(path) => PathBuf::from_str(&path).unwrap_or(default),
        Err(_e) => default
    }
}

fn init_cfg_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(config_path())
}

pub fn init_logset() -> BoxResult<(LogSet, String)> {
    let cfg_dir = config_path().join("config.yaml");
    let cfg_path = cfg_dir
            .to_str()
            .expect("could not find configuration directory!");

    init_cfg_dir()?;
    let logset = LogSet::from_path(cfg_path)?;

    return Ok((logset, cfg_path.into()));
}

pub fn command_line() -> BoxResult<Interface> {
    let options = Opts::parse();
    let (mut logset, cfg_path) = init_logset()?;

    let exit = match &options.subcmd {
        Some(subcmd) => {
            match &subcmd {
                SubCommand::Add(add) => add_cmd(&add, &mut logset)?,
                SubCommand::List(list) => list_cmd(&list, &mut logset)?,
                SubCommand::Delete(delete) => delete_cmd(&delete, &mut logset)?,
                SubCommand::AddReTrigger(re) => add_re_trigger(&re, &mut logset)?,
                SubCommand::ListTrigger(lt) => list_trigger(&lt, &mut logset)?,
                SubCommand::DeleteTrigger(dt) => delete_trigger(&dt, &mut logset)?
            }
        },
        _ => false
    };

    logset.to_file(&cfg_path)?;
    if exit {
        std::process::exit(0);
    }

    Ok(Interface {
        logset,
        options,
        cfg_path: cfg_path.into()
    })
}

pub fn add_cmd(add: &Add, logset: &mut LogSet) -> BoxResult<bool> {
    match add.logtype.as_str() {
        "local" => {
            logset.logs.push(
                Logfile::new(&add.name,
                    Box::new(FileDataSource::new(&add.location, add.line_limit)),
                    Task::from_str(true, &add.refresh_time, Box::new(ClockTimeSource))?
            ))
        }
        _ => println!("Invalid logfile type!")
    }
    Ok(true)
}

pub fn list_cmd(_list: &List, logset: &mut LogSet) -> BoxResult<bool> {
    for (i, log) in logset.logs.iter().enumerate() {
        println!("{}: {}", i, log.name);
    }

    Ok(true)
}

pub fn delete_cmd(delete: &Delete, logset: &mut LogSet) -> BoxResult<bool> {
    logset.remove(delete.index);
    Ok(true)
}

pub fn add_re_trigger(re: &AddReTrigger, logset: &mut LogSet) -> BoxResult<bool> {
    if re.index >= logset.len() {
        println!("Index out of bounds!");
    } else {
        let log = &mut logset.logs[re.index];

        let trigger_type = match re.trigger_type.as_str() {
            "success" => TriggerType::Success,
            "warning" => TriggerType::Warning,
            "error" => TriggerType::Error,
            _ => {
                println!("Unknown trigger type!");
                return Ok(true);
            }
        };


        log.push(Box::new(RegexTrigger::new(&re.name, &re.desc, trigger_type, &re.regex)));

    }
    Ok(true)
}

pub fn list_trigger(lt: &ListTrigger, logset: &mut LogSet) -> BoxResult<bool> {
    if lt.index >= logset.len() {
        println!("Index out of bounds!");
    } else {
        let log = &mut logset.logs[lt.index];

        for (i, trigger) in log.triggers.iter().enumerate() {
            println!("{}: {} - {}", i, trigger.name(), trigger.description());
        }
    }
    Ok(true)
}

pub fn delete_trigger(dt: &DeleteTrigger, logset: &mut LogSet) -> BoxResult<bool> {
    if dt.log_index >= logset.len() {
        println!("Index out of bounds!");
    } else {
        let log = &mut logset.logs[dt.log_index];
        log.remove(dt.trigger_index);
    }
    Ok(true)
}
