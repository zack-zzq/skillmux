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
    let (repo, r#ref) = repo
        .split_once("@")
        .map(|(r, rf)| (r.to_string(), rf.to_string()))
        .unwrap_or((repo, "HEAD".into()));
    Some(GitHubSource {
        url: format!("https://github.com/{owner}/{repo}"),
        owner,
        repo,
        r#ref,
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

pub fn repo_description(owner: &str, repo: &str) -> Option<String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}");
    let c = reqwest::blocking::Client::new();
    let r = c.get(url).header("user-agent", "skillmux").send().ok()?;
    let v: serde_json::Value = r.json().ok()?;
    v.get("description")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parse_supports_prefix_and_ref() {
        let s = parse("gh:owner/repo@main").expect("parsed");
        assert_eq!(s.owner, "owner");
        assert_eq!(s.repo, "repo");
        assert_eq!(s.r#ref, "main");

        let s2 = parse("github:owner/repo").expect("parsed");
        assert_eq!(s2.r#ref, "HEAD");
    }
}
