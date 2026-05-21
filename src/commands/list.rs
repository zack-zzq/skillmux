use crate::{config::target_skill_dir, config::Config, storage::SkillStorage};
use anyhow::Result;
pub fn run(cfg: &Config, json: bool) -> Result<()> {
    let mut all = vec![];
    for t in &cfg.install.targets {
        let s = SkillStorage::new(target_skill_dir(t));
        for n in s.list().unwrap_or_default() {
            all.push((t.clone(), n));
        }
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&all)?);
    } else {
        for (t, n) in all {
            println!("{t}: {n}");
        }
    }
    Ok(())
}
