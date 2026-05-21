use crate::{
    config::{target_skill_dir, Config},
    storage::SkillStorage,
};
use anyhow::Result;
pub fn run(cfg: &Config, skill: &str) -> Result<()> {
    for t in &cfg.install.targets {
        let s = SkillStorage::new(target_skill_dir(t));
        let _ = s.remove(skill);
    }
    println!("removed {skill}");
    Ok(())
}
