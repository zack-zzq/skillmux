use crate::{api::ApiClient, config::Config};
use anyhow::Result;
pub fn run(_api: &ApiClient, _cfg: &Config, skill: Option<String>, all: bool) -> Result<()> {
    println!("update {:?} all={}", skill, all);
    Ok(())
}
