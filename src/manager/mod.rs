pub mod cli;
pub mod cmd;

use crate::manager::cmd::init::InitOptions;
use crate::manager::cmd::mode::Mode;
use anyhow::Result;
use clap::Parser;
use cli::{Args, Cmd, GhqCmd, GitCmd, ModeCmd, OwnersCmd};

pub fn run() -> Result<()> {
    let args = Args::parse();
    match args.cmd {
        Cmd::Init { mode, no_prompt } => {
            let parsed_mode = match mode.as_deref() {
                Some(m) => Some(Mode::parse(m)?),
                None => None,
            };
            cmd::init::run(InitOptions {
                mode: parsed_mode,
                no_prompt,
            })
        }
        Cmd::Login { profile } => cmd::login::run(profile),
        Cmd::List => cmd::list::run(),
        Cmd::Which => cmd::which::run(),
        Cmd::Use { profile } => cmd::use_cmd::run(&profile),
        Cmd::Unuse => cmd::unuse::run(),
        Cmd::Config => cmd::config_cmd::run(),
        Cmd::Doctor => cmd::doctor::run(),
        Cmd::Import { profile } => cmd::import::run(&profile),
        Cmd::Owners(OwnersCmd::List) => cmd::owners::list(),
        Cmd::Owners(OwnersCmd::Map { owner, profile }) => cmd::owners::map(&owner, &profile),
        Cmd::Owners(OwnersCmd::Unmap { owner }) => cmd::owners::unmap(&owner),
        Cmd::Ghq(GhqCmd::List) => cmd::ghq::list(),
        Cmd::Ghq(GhqCmd::Map { owner, profile }) => cmd::ghq::map(&owner, &profile),
        Cmd::Ghq(GhqCmd::Unmap { owner }) => cmd::ghq::unmap(&owner),
        Cmd::Ghq(GhqCmd::Enable) => cmd::ghq::enable(),
        Cmd::Ghq(GhqCmd::Disable) => cmd::ghq::disable(),
        Cmd::Ghq(GhqCmd::Root { path }) => cmd::ghq::set_root(path),
        Cmd::Ghq(GhqCmd::Install) => cmd::ghq::install(),
        Cmd::Git(GitCmd::Show) => cmd::git::show(),
        Cmd::Git(GitCmd::Enable) => cmd::git::enable(),
        Cmd::Git(GitCmd::Disable) => cmd::git::disable(),
        Cmd::Git(GitCmd::Remote { name }) => cmd::git::set_remote(&name),
        Cmd::Mode(ModeCmd::Show) => cmd::mode::show(),
        Cmd::Mode(ModeCmd::Set { mode }) => cmd::mode::set(&mode),
    }
}
