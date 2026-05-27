use crate::{
    config::{target_skill_dir, Config},
    storage::SkillStorage,
};
use anyhow::Result;

pub fn run(cfg: &Config, json: bool) -> Result<()> {
    let mut rows = vec![];
    for t in &cfg.install.targets {
        let s = SkillStorage::new(target_skill_dir(t));
        for n in s.list().unwrap_or_default() {
            if let Some(info) = s.load_info(&n) {
                let src = info
                    .source
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("legacy");
                rows.push(serde_json::json!({"target":t,"name":info.name,"source":src,"version":info.version,"commit":info.source.get("commit")}));
            }
        }
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        for r in rows {
            println!(
                "{:<10} {:<24} {:<8} {}",
                r["target"], r["name"], r["source"], r["version"]
            );
        }
    }
    Ok(())
}
