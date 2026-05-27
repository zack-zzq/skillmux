use crate::{
    api::ApiClient,
    commands::install,
    config::{target_skill_dir, Config},
    storage::SkillStorage,
};
use anyhow::{anyhow, Result};
use serde_json::Value;

fn source_type(info: &Value) -> Option<&str> {
    info.get("type")?.as_str()
}

pub fn run(
    api: &ApiClient,
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
                targets.push((n, info.source));
            }
        }
    }
    targets.sort_by(|a, b| a.0.cmp(&b.0));
    targets.dedup_by(|a, b| a.0 == b.0);
    for (name, info) in targets {
        match source_type(&info) {
            Some("github") => {
                let url = info
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("missing source.url for GitHub skill {name}"))?;
                let subdir = info
                    .get("subdir")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                let source_ref = ref_name
                    .clone()
                    .or_else(|| info.get("ref").and_then(|v| v.as_str()).map(str::to_string));
                install::run(
                    api,
                    cfg,
                    url,
                    None,
                    source_ref,
                    subdir,
                    Some(name),
                    true,
                    true,
                    false,
                )?;
            }
            _ => install::run(
                api,
                cfg,
                &name,
                None,
                ref_name.clone(),
                None,
                None,
                true,
                true,
                false,
            )?,
        }
    }
    Ok(())
}
