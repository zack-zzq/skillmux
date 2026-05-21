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
        Cmd::Search {
            keyword,
            limit,
            page,
            json,
        } => search::run(api, keyword, limit, page, json),
        Cmd::Install {
            skill,
            version,
            force,
            json,
        } => install::run(api, cfg, &skill, version, force, json),
        Cmd::List { json } => list::run(cfg, json),
        Cmd::Update { skill, all } => update::run(api, cfg, skill, all),
        Cmd::Remove { skill } => remove::run(cfg, &skill),
        Cmd::Config { sub } => config::run(cfg, sub),
    }
}
