use super::logset::LogSet;
use std::path::PathBuf;
use super::dirs;
use super::error::BoxResult;
use super::clap::{AppSettings, Clap};

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
}

// TODO allow user to move config path?
pub fn config_path() -> PathBuf {
    dirs::home_dir().expect("Unable to get home directory!")
        .join(".config/minutecat")
}

fn init_cfg_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(config_path())
}

pub fn command_line() -> BoxResult<Interface> {
    let options = Opts::parse();

    let cfg_dir = config_path().join("config.yaml");
    let cfg_path = cfg_dir
            .to_str()
            .expect("could not find configuration directory!");

    init_cfg_dir()?;
    let logset = LogSet::from_path(cfg_path)?;

    Ok(Interface {
        logset,
        options,
        cfg_path: cfg_path.into()
    })
}
