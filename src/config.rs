use crate::api::ApiClient;
use anyhow::Result;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

pub const ALL_TARGETS: &[&str] = &["codex", "qoder", "qoderwork", "kiro", "workbuddy"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api: Api,
    pub install: Install,
    #[serde(skip)]
    path: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Api {
    pub endpoint: String,
    pub timeout: u64,
    pub token: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Install {
    pub targets: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api: Api {
                endpoint: "https://skills.kingdee.com/api".into(),
                timeout: 30,
                token: None,
            },
            install: Install {
                targets: ALL_TARGETS.iter().map(|s| s.to_string()).collect(),
            },
            path: config_path(None).unwrap(),
        }
    }
}

pub fn config_path(custom: Option<&str>) -> Result<PathBuf> {
    if let Some(c) = custom {
        return Ok(PathBuf::from(c));
    }
    Ok(BaseDirs::new()
        .unwrap()
        .home_dir()
        .join(".config/skillhub/config.yaml"))
}
impl Config {
    pub fn load(custom: Option<&str>) -> Result<Self> {
        let path = config_path(custom)?;
        if let Some(p) = path.parent() {
            fs::create_dir_all(p)?;
        }
        let mut c = Self {
            path: path.clone(),
            ..Default::default()
        };
        if path.exists() {
            let t = fs::read_to_string(&path)?;
            let user: serde_yaml::Value = serde_yaml::from_str(&t)?;
            let mut base = serde_yaml::to_value(&c)?;
            merge(&mut base, user);
            c = serde_yaml::from_value(base)?;
            c.path = path;
        }
        Ok(c)
    }
    pub fn save(&self) -> Result<()> {
        fs::write(&self.path, serde_yaml::to_string(self)?)?;
        Ok(())
    }
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "api.endpoint" => Some(self.api.endpoint.clone()),
            "api.timeout" => Some(self.api.timeout.to_string()),
            "api.token" => self.api.token.clone(),
            "install.targets" => Some(self.install.targets.join(",")),
            _ => None,
        }
    }
    pub fn set(&mut self, key: &str, val: &str) {
        match key {
            "api.endpoint" => self.api.endpoint = val.into(),
            "api.timeout" => self.api.timeout = val.parse().unwrap_or(30),
            "api.token" => self.api.token = Some(val.into()),
            "install.targets" => self.install.targets = parse_targets(val),
            _ => {}
        }
    }
    pub fn resolve_token(&self, cli_token: Option<String>) -> String {
        cli_token
            .or_else(|| env::var("KDSKILLHUB_TOKEN").ok())
            .or_else(|| self.api.token.clone())
            .unwrap_or_else(ApiClient::default_token)
    }
}
fn merge(a: &mut serde_yaml::Value, b: serde_yaml::Value) {
    match (a, b) {
        (serde_yaml::Value::Mapping(a), serde_yaml::Value::Mapping(b)) => {
            for (k, v) in b {
                merge(a.entry(k).or_insert(serde_yaml::Value::Null), v)
            }
        }
        (a, b) => *a = b,
    }
}
pub fn parse_targets(v: &str) -> Vec<String> {
    v.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| ALL_TARGETS.contains(&s.as_str()))
        .collect()
}
pub fn install_check_path(t: &str) -> PathBuf {
    let h = BaseDirs::new().unwrap().home_dir().to_path_buf();
    match t {
        "codex" => h.join(".codex/skills"),
        "workbuddy" => h.join(".workbuddy/skills"),
        "qoder" => h.join(".qoder"),
        "qoderwork" => h.join(".qoderwork"),
        "kiro" => h.join(".kiro"),
        _ => h,
    }
}
pub fn target_skill_dir(t: &str) -> PathBuf {
    let h = BaseDirs::new().unwrap().home_dir().to_path_buf();
    match t {
        "codex" => h.join(".codex/skills"),
        _ => h.join(format!(".{t}/skills")),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn targets() {
        assert_eq!(parse_targets("codex,bad,qoder"), vec!["codex", "qoder"]);
    }
    #[test]
    fn token_priority() {
        let mut c = Config::default();
        c.api.token = Some("cfg".into());
        std::env::set_var("KDSKILLHUB_TOKEN", "env");
        assert_eq!(c.resolve_token(Some("cli".into())), "cli");
        assert_eq!(c.resolve_token(None), "env");
        std::env::remove_var("KDSKILLHUB_TOKEN");
        assert_eq!(c.resolve_token(None), "cfg");
    }
}
