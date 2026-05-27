use crate::{
    config::{target_skill_dir, Config},
    storage::SkillStorage,
};
use anyhow::Result;
pub fn run(cfg: &Config, skill: &str) -> Result<()> {
    let mut removed = 0usize;
    for t in &cfg.install.targets {
        let s = SkillStorage::new(target_skill_dir(t));
        let p = s.skill_path(skill);
        if p.exists() {
            s.remove(skill)?;
            removed += 1;
        }
    }
    if removed > 0 {
        println!("Removed {skill} from {removed} target(s).");
    } else {
        println!("Skill `{skill}` is not installed.");
    }
    Ok(())
}
