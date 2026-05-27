use crate::{
    config::target_skill_dir, config::Config, storage::InstalledSkill, storage::SkillStorage,
};
use anyhow::Result;
pub fn run(cfg: &Config, json: bool) -> Result<()> {
    let mut all: Vec<(String, InstalledSkill)> = vec![];
    for t in &cfg.install.targets {
        let s = SkillStorage::new(target_skill_dir(t));
        for n in s.list().unwrap_or_default() {
            if let Some(info) = s.load_info(&n) {
                all.push((t.clone(), info));
            }
        }
    }
    all.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.name.cmp(&b.1.name)));
    if json {
        println!("{}", serde_json::to_string_pretty(&all)?);
    } else {
        if all.is_empty() {
            println!("No skills installed by kdskillhub.");
        } else {
            for (t, info) in all {
                println!(
                    "{t:<10} {name:<36} v{version}",
                    name = info.name,
                    version = info.version
                );
            }
        }
    }
    Ok(())
}
