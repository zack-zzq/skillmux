use crate::{
    config::{target_skill_dir, Config},
    storage::SkillStorage,
};
use anyhow::{anyhow, Result};
use directories::BaseDirs;

pub fn run(cfg: &Config, skill: &str, purge: bool) -> Result<()> {
    let mut removed_targets = Vec::new();

    for t in &cfg.install.targets {
        let s = SkillStorage::new(target_skill_dir(t));

        if s.remove(skill)? {
            removed_targets.push(t.clone());
        }
    }

    if removed_targets.is_empty() {
        return Err(anyhow!("skill not installed: {skill}"));
    }

    if purge {
        let c = BaseDirs::new()
            .ok_or_else(|| anyhow!("failed to resolve user cache directory"))?
            .cache_dir()
            .join("kdskillhub/github");

        if c.exists() {
            std::fs::remove_dir_all(c)?;
        }
    }

    if removed_targets.len() == 1 {
        println!("Removed skill: {skill}");
    } else {
        println!(
            "Removed skill: {skill} from {} targets",
            removed_targets.len()
        );
    }

    Ok(())
}