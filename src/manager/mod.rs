pub mod cli;
pub mod cmd;

use anyhow::Result;
use clap::Parser;
use cli::{Args, Cmd};

pub fn run() -> Result<()> {
    let args = Args::parse();
    match args.cmd {
        Cmd::Init => cmd::init::run(),
        Cmd::Login { profile } => cmd::login::run(profile),
        Cmd::List => cmd::list::run(),
        Cmd::Which => cmd::which::run(),
        Cmd::Use { profile } => cmd::use_cmd::run(&profile),
        Cmd::Unuse => cmd::unuse::run(),
        Cmd::Config => cmd::config_cmd::run(),
        Cmd::Doctor => cmd::doctor::run(),
        Cmd::Import { profile } => cmd::import::run(&profile),
    }
}
