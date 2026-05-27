use crate::{
    api::ApiClient,
    commands::install,
    config::{target_skill_dir, Config},
    sources::clawhub::ClawHubSource,
    storage::SkillStorage,
};
use anyhow::Result;
use serde_json::Value;

fn source_name(v: &Value) -> &str {
    match v {
        Value::String(s) if !s.trim().is_empty() => s,
        Value::Object(map) => map
            .get("source")
            .or_else(|| map.get("type"))
            .and_then(|x| x.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("kingdee"),
        _ => "kingdee",
    }
}

pub fn run(
    api: &ApiClient,
    claw: &ClawHubSource,
    cfg: &Config,
    skill: Option<String>,
    all: bool,
    ref_name: Option<String>,
) -> Result<()> {
    let mut targets = vec![];
    for t in &cfg.install.targets {
        let s = SkillStorage::new(target_skill_dir(t));
        for n in s.list().unwrap_or_default() {
            if !(all || skill.as_deref() == Some(&n)) {
                continue;
            }
            if let Some(info) = s.load_info(&n) {
                targets.push((n, source_name(&info.source).to_string()));
            }
        }
    }
    targets.sort();
    targets.dedup();
    for (name, src) in targets {
        install::run(
            api,
            claw,
            &src,
            cfg,
            &format!("{}:{}", src, name),
            None,
            ref_name.clone(),
            None,
            None,
            true,
            true,
            false,
        )?;
    }
    Ok(())
}
