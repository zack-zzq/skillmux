use crate::{config::{target_skill_dir, Config}, storage::SkillStorage};
use anyhow::Result;
use directories::BaseDirs;

pub fn run(cfg: &Config, skill: &str, purge: bool) -> Result<()> {
    for t in &cfg.install.targets {
        let s = SkillStorage::new(target_skill_dir(t));
        s.remove(skill)?;
    }
    if purge {
        let c = BaseDirs::new().unwrap().cache_dir().join("kdskillhub/github");
        if c.exists() { std::fs::remove_dir_all(c)?; }
    }
    Ok(())
}
