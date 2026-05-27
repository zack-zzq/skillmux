use crate::{api::ApiClient, commands::install, config::{target_skill_dir, Config}, storage::SkillStorage};
use anyhow::Result;

pub fn run(api: &ApiClient, cfg: &Config, skill: Option<String>, all: bool, ref_name: Option<String>) -> Result<()> {
    let mut names = vec![];
    for t in &cfg.install.targets {
        let s = SkillStorage::new(target_skill_dir(t));
        for n in s.list().unwrap_or_default() {
            if all || skill.as_deref() == Some(&n) { names.push(n); }
        }
    }
    names.sort(); names.dedup();
    for n in names {
        install::run(api, cfg, &n, None, ref_name.clone(), None, None, true, true, false)?;
    }
    Ok(())
}
