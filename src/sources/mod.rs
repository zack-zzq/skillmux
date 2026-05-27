pub mod clawhub;
pub mod github;
pub mod kingdee;

use anyhow::Result;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteSkillVersion {
    pub version: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteSkill {
    pub name: String,
    pub slug: String,
    pub version: Option<String>,
    pub canonical_url: Option<String>,
    pub source: String,
    pub description: Option<String>,
    #[serde(default)]
    pub meta: serde_json::Value,
}

pub trait SkillSource {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    fn search(&self, keyword: Option<String>, limit: u32, page: u32) -> Result<Vec<RemoteSkill>>;
    fn resolve(&self, slug: &str) -> Result<RemoteSkill>;
    #[allow(dead_code)]
    fn versions(&self, slug: &str) -> Result<Vec<RemoteSkillVersion>>;
    fn download(&self, slug: &str, version: Option<&str>) -> Result<Vec<u8>>;
    fn pre_install_check(&self, _slug: &str) -> Result<()> {
        Ok(())
    }
}
