use crate::{
    api::ApiClient,
    commands::install,
    config::{target_skill_dir, Config},
    sources::clawhub::ClawHubSource,
    storage::SkillStorage,
};
use anyhow::Result;

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
                let src_type = info
                    .source
                    .get("type")
                    .and_then(|v| v.as_str())
                    .or_else(|| info.source.as_str())
                    .unwrap_or("kingdee")
                    .to_string();
                let source_skill = if src_type == "github" {
                    let owner = info.source.get("owner").and_then(|v| v.as_str());
                    let repo = info.source.get("repo").and_then(|v| v.as_str());
                    let rf = ref_name
                        .as_deref()
                        .or_else(|| info.source.get("ref").and_then(|v| v.as_str()))
                        .unwrap_or("HEAD");
                    match (owner, repo) {
                        (Some(owner), Some(repo)) => format!("github:{owner}/{repo}@{rf}"),
                        _ => format!("{}:{}", src_type, n),
                    }
                } else {
                    format!("{}:{}", src_type, n)
                };
                targets.push((n, src_type, source_skill));
            }
        }
    }
    targets.sort();
    targets.dedup();
    for (name, src, source_skill) in targets {
        println!("Updating {name} from {src}...");
        install::run(
            api,
            claw,
            &src,
            cfg,
            &source_skill,
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
