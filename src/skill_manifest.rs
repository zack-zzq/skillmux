use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct SkillManifest {
    pub name: String,
    pub description: Option<String>,
}

pub fn parse_skill_md(content: &str) -> Result<SkillManifest> {
    let fm = content
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|x| x.0))
        .ok_or_else(|| anyhow!("SKILL.md must contain frontmatter"))?;
    let mut name = None;
    let mut description = None;
    for line in fm.lines() {
        if let Some((k, v)) = line.split_once(':') {
            let val = v.trim().trim_matches('"').trim_matches('\'');
            match k.trim() {
                "name" => name = Some(val.to_string()),
                "description" => description = Some(val.to_string()),
                _ => {}
            }
        }
    }
    Ok(SkillManifest {
        name: name.ok_or_else(|| anyhow!("frontmatter name is required"))?,
        description,
    })
}
