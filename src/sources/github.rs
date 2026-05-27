use anyhow::{anyhow, Result};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GitHubSource {
    pub owner: String,
    pub repo: String,
    pub url: String,
    pub r#ref: String,
    pub subdir: Option<String>,
}

pub fn parse(input: &str) -> Option<GitHubSource> {
    let path = input
        .strip_prefix("gh:")
        .or_else(|| input.strip_prefix("github:"))
        .or_else(|| input.strip_prefix("https://github.com/"))?;
    let mut p = path.split('/');
    let owner = p.next()?.to_string();
    let repo = p.next()?.trim_end_matches(".git").to_string();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some(GitHubSource {
        url: format!("https://github.com/{owner}/{repo}"),
        owner,
        repo,
        r#ref: "HEAD".into(),
        subdir: None,
    })
}

pub fn cache_key(owner: &str, repo: &str, r: &str) -> String {
    format!("{owner}/{repo}/{}", r.replace('/', "_"))
}

pub fn validate_skill_root(root: &std::path::Path) -> Result<()> {
    if !root.join("SKILL.md").exists() {
        return Err(anyhow!("missing SKILL.md in source directory"));
    }
    Ok(())
}
