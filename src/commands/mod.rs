pub mod config;
pub mod install;
pub mod list;
pub mod remove;
pub mod search;
pub mod update;

use crate::{api::ApiClient, cli::Cmd, config::Config};
use anyhow::Result;

pub fn dispatch(cmd: Cmd, cfg: &mut Config, api: &ApiClient) -> Result<()> {
    match cmd {
        Cmd::Search { keyword, limit, page, json } => search::run(api, keyword, limit, page, json),
        Cmd::Install { skill, version, ref_name, subdir, as_name, yes, force, json } => install::run(api, cfg, &skill, version, ref_name, subdir, as_name, yes, force, json),
        Cmd::List { json } => list::run(cfg, json),
        Cmd::Update { skill, all, ref_name } => update::run(api, cfg, skill, all, ref_name),
        Cmd::Remove { skill, purge } => remove::run(cfg, &skill, purge),
        Cmd::Config { sub } => config::run(cfg, sub),
    }
}
