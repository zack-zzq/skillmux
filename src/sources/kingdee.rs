use crate::{
    api::{extract_list, ApiClient},
    sources::{RemoteSkill, RemoteSkillVersion, SkillSource},
};
use anyhow::{anyhow, Result};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KingdeeSource {
    pub id: i64,
    pub version: String,
}

impl SkillSource for ApiClient {
    fn name(&self) -> &'static str {
        "kingdee"
    }
    fn search(&self, keyword: Option<String>, limit: u32, page: u32) -> Result<Vec<RemoteSkill>> {
        let data = crate::api::ApiClient::search(self, keyword, page, limit)?;
        Ok(extract_list(&data)?
            .into_iter()
            .map(|s| RemoteSkill {
                name: s.name.clone(),
                slug: s.name,
                version: s.current_version.or(s.version),
                canonical_url: None,
                source: "kingdee".into(),
                meta: serde_json::json!({"id":s.id}),
            })
            .collect())
    }
    fn resolve(&self, slug: &str) -> Result<RemoteSkill> {
        <ApiClient as SkillSource>::search(self, Some(slug.into()), 20, 1)?
            .into_iter()
            .find(|s| s.slug == slug)
            .ok_or_else(|| anyhow!("not found"))
    }
    fn versions(&self, slug: &str) -> Result<Vec<RemoteSkillVersion>> {
        let one = self.resolve(slug)?;
        Ok(one
            .version
            .map(|v| vec![RemoteSkillVersion { version: v }])
            .unwrap_or_default())
    }
    fn download(&self, slug: &str, version: Option<&str>) -> Result<Vec<u8>> {
        let one = self.resolve(slug)?;
        let id = one
            .meta
            .get("id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow!("missing id"))?;
        let ver = version
            .map(|s| s.to_string())
            .or(one.version)
            .ok_or_else(|| anyhow!("no version"))?;
        crate::api::ApiClient::download(self, id, &ver)
    }
}
