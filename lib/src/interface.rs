use super::clap::{Args, Parser, Subcommand};
use super::command::*;
use super::dirs;
use super::error::Error;
use super::logset::LogSet;
use super::trigger::{Trigger, TriggerType};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;

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
    pub cfg_path: String,
}

#[derive(Parser)]
#[clap(version = "0.1.0", author = "Lukas Krickl <lukas@krickl.dev>")]
pub struct Opts {
    #[clap(subcommand)]
    subcmd: Option<SubCommand>,
}

#[derive(Subcommand)]
pub enum SubCommand {
    Add(Add),

    List(List),

    Delete(Delete),

    AddReTrigger(AddReTrigger),

    ListTrigger(ListTrigger),

    DeleteTrigger(DeleteTrigger),
}

#[derive(Args)]
pub struct Add {
    name: String,
    location: String,
    line_limit: usize,
    logtype: FileType,
    refresh_time: String,
}

#[derive(Args)]
pub struct List;

#[derive(Args)]
pub struct Delete {
    pub index: usize,
}

#[derive(Args)]
pub struct AddReTrigger {
    pub index: usize,
    pub name: String,
    pub desc: String,
    pub trigger_type: TriggerType,
    pub regex: String,
    #[clap(long)]
    pub invert: bool,
}

#[derive(Args)]
pub struct ListTrigger {
    pub index: usize,
}

#[derive(Args)]
pub struct DeleteTrigger {
    pub log_index: usize,
    pub trigger_index: usize,
}

// TODO allow user to move config path?
pub fn config_path() -> PathBuf {
    let default = dirs::home_dir()
        .expect("Unable to get home directory!")
        .join(".config/minutecat");
    match env::var("MINUTECATDIR") {
        Ok(path) => PathBuf::from_str(&path).unwrap_or(default),
        Err(_e) => default,
    }
}

fn init_cfg_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(config_path())
}

pub fn init_logset() -> Result<(LogSet, String), Error> {
    let cfg_dir = config_path().join("config.yaml");
    let cfg_path = cfg_dir
        .to_str()
        .expect("could not find configuration directory!");

    init_cfg_dir()?;
    let logset = LogSet::from_path(cfg_path)?;

    Ok((logset, cfg_path.into()))
}

pub fn command_line() -> Result<Interface, Error> {
    let options = Opts::parse();
    let (mut logset, cfg_path) = init_logset()?;

    let exit = match &options.subcmd {
        Some(subcmd) => match &subcmd {
            SubCommand::Add(add) => add_cmd(add, &mut logset)?,
            SubCommand::List(list) => list_cmd(list, &mut logset)?,
            SubCommand::Delete(delete) => delete_cmd(delete, &mut logset)?,
            SubCommand::AddReTrigger(re) => add_re_trigger(re, &mut logset)?,
            SubCommand::ListTrigger(lt) => list_trigger(lt, &mut logset)?,
            SubCommand::DeleteTrigger(dt) => delete_trigger(dt, &mut logset)?,
        },
        _ => false,
    };

    logset.to_file(&cfg_path)?;
    if exit {
        std::process::exit(0);
    }

    Ok(Interface {
        logset,
        options,
        cfg_path,
    })
}

pub fn add_cmd(add: &Add, logset: &mut LogSet) -> Result<bool, Error> {
    match add.logtype {
        FileType::Local => {
            let mut cmd = AddFileCommand::new(
                &add.name,
                &add.location,
                add.line_limit,
                &add.refresh_time,
                FileType::Local,
            );
            cmd.execute(logset)?;
        }
        FileType::Http => {
            let mut cmd = AddFileCommand::new(
                &add.name,
                &add.location,
                add.line_limit,
                &add.refresh_time,
                FileType::Http,
            );
            cmd.execute(logset)?;
        }
    }
    Ok(true)
}

pub fn list_cmd(_list: &List, logset: &mut LogSet) -> Result<bool, Error> {
    for (i, log) in logset.logs.iter().enumerate() {
        println!("{}: {}", i, log.name);
    }

    Ok(true)
}

pub fn delete_cmd(delete: &Delete, logset: &mut LogSet) -> Result<bool, Error> {
    let mut cmd = DeleteLogfileCommand::new(delete.index);
    cmd.execute(logset)?;
    Ok(true)
}

pub fn add_re_trigger(re: &AddReTrigger, logset: &mut LogSet) -> Result<bool, Error> {
    if re.index >= logset.len() {
        println!("Index out of bounds!");
    } else {
        let log = &mut logset.logs[re.index];

        if re.trigger_type == TriggerType::NoEvent {
            println!("Unknown trigger type!");
            return Ok(true);
        }

        let mut cmd =
            AddRegexTriggerCommand::new(&re.name, &re.desc, re.trigger_type, &re.regex, re.invert);
        cmd.execute(log)?;
    }
    Ok(true)
}

pub fn list_trigger(lt: &ListTrigger, logset: &mut LogSet) -> Result<bool, Error> {
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

pub fn delete_trigger(dt: &DeleteTrigger, logset: &mut LogSet) -> Result<bool, Error> {
    if dt.log_index >= logset.len() {
        println!("Index out of bounds!");
    } else {
        let log = &mut logset.logs[dt.log_index];

        let mut cmd = RemoveTriggerCommand::new(dt.log_index);
        cmd.execute(log)?;
    }
    Ok(true)
}
