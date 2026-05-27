use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct SkillManifest {
    pub name: String,
    #[allow(dead_code)]
    pub description: Option<String>,
}

pub fn parse_skill_md(content: &str) -> Result<SkillManifest> {
    let normalized = normalize_markdown(content);

    let frontmatter = extract_frontmatter(&normalized)
        .ok_or_else(|| anyhow!("SKILL.md must contain frontmatter"))?;

    let mut name = None;
    let mut description = None;

    for line in frontmatter.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let value = value
                .trim()
                .trim_matches('"')
                .trim_matches('\'');

            match key.trim() {
                "name" => {
                    if !value.is_empty() {
                        name = Some(value.to_string());
                    }
                }
                "description" => {
                    if !value.is_empty() {
                        description = Some(value.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    Ok(SkillManifest {
        name: name.ok_or_else(|| anyhow!("frontmatter name is required"))?,
        description,
    })
}

fn normalize_markdown(content: &str) -> String {
    content
        .trim_start_matches('\u{feff}')
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

fn extract_frontmatter(content: &str) -> Option<&str> {
    let mut lines = content.lines();

    let first = lines.next()?.trim();
    if first != "---" {
        return None;
    }

    let start = content.find('\n')? + 1;
    let rest = &content[start..];

    for (idx, line) in rest.lines().enumerate() {
        if line.trim() == "---" {
            let mut byte_end = 0;

            for prior_line in rest.lines().take(idx) {
                byte_end += prior_line.len();
                byte_end += 1;
            }

            return Some(&rest[..byte_end]);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lf_frontmatter() {
        let content = "---\nname: demo\ndescription: test skill\n---\n# Demo\n";
        let manifest = parse_skill_md(content).expect("parse");

        assert_eq!(manifest.name, "demo");
        assert_eq!(manifest.description.as_deref(), Some("test skill"));
    }

    #[test]
    fn parse_crlf_frontmatter() {
        let content = "---\r\nname: demo\r\ndescription: test skill\r\n---\r\n# Demo\r\n";
        let manifest = parse_skill_md(content).expect("parse");

        assert_eq!(manifest.name, "demo");
        assert_eq!(manifest.description.as_deref(), Some("test skill"));
    }

    #[test]
    fn parse_bom_frontmatter() {
        let content = "\u{feff}---\nname: demo\n---\n# Demo\n";
        let manifest = parse_skill_md(content).expect("parse");

        assert_eq!(manifest.name, "demo");
        assert_eq!(manifest.description, None);
    }

    #[test]
    fn parse_quoted_values() {
        let content = "---\nname: \"demo\"\ndescription: 'test skill'\n---\n";
        let manifest = parse_skill_md(content).expect("parse");

        assert_eq!(manifest.name, "demo");
        assert_eq!(manifest.description.as_deref(), Some("test skill"));
    }

    #[test]
    fn reject_missing_frontmatter() {
        let content = "# Demo\n";
        let err = parse_skill_md(content).expect_err("should fail");

        assert!(err.to_string().contains("frontmatter"));
    }

    #[test]
    fn reject_missing_name() {
        let content = "---\ndescription: test skill\n---\n";
        let err = parse_skill_md(content).expect_err("should fail");

        assert!(err.to_string().contains("name"));
    }
}