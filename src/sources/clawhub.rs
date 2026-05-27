use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde_json::Value;

use super::{RemoteSkill, RemoteSkillVersion, SkillSource};

#[derive(Clone)]
pub struct ClawHubSource {
    pub base: String,
    client: Client,
}
impl ClawHubSource {
    pub fn new(base: Option<String>, timeout: u64) -> Result<Self> {
        Ok(Self {
            base: base
                .unwrap_or_else(|| "https://clawhub.ai".into())
                .trim_end_matches('/')
                .to_string(),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(timeout))
                .build()?,
        })
    }
    fn get_json(&self, path: &str, params: &[(&str, String)]) -> Result<Value> {
        let r = self
            .client
            .get(format!("{}{}", self.base, path))
            .query(params)
            .send()?;
        if r.status().as_u16() == 429 {
            return Err(anyhow!("rate limited (429)"));
        }
        Ok(r.json()?)
    }
    fn map_skill(&self, v: &Value) -> RemoteSkill {
        let slug = v
            .get("slug")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string();
        let name = v
            .get("name")
            .and_then(|x| x.as_str())
            .unwrap_or(slug.as_str())
            .to_string();
        let version = v
            .get("version")
            .or_else(|| v.get("latestVersion"))
            .or_else(|| v.get("tag"))
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());
        RemoteSkill {
            name,
            slug: slug.clone(),
            version,
            canonical_url: Some(format!("{}/skills/{}", self.base, slug)),
            source: "clawhub".into(),
            description: v
                .get("description")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            meta: v.clone(),
        }
    }
}
impl SkillSource for ClawHubSource {
    fn name(&self) -> &'static str {
        "clawhub"
    }
    fn search(&self, keyword: Option<String>, limit: u32, _page: u32) -> Result<Vec<RemoteSkill>> {
        let q = keyword.unwrap_or_default();
        let v = self.get_json(
            "/api/v1/search",
            &[
                ("q", q),
                ("limit", limit.to_string()),
                ("nonSuspiciousOnly", "true".into()),
            ],
        )?;
        let arr = v
            .get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .or_else(|| v.as_array().cloned())
            .unwrap_or_default();
        Ok(arr.iter().map(|i| self.map_skill(i)).collect())
    }
    fn resolve(&self, slug: &str) -> Result<RemoteSkill> {
        let v = self.get_json(&format!("/api/v1/skills/{slug}"), &[])?;
        Ok(self.map_skill(v.get("data").unwrap_or(&v)))
    }
    fn versions(&self, slug: &str) -> Result<Vec<RemoteSkillVersion>> {
        let v = self.get_json(&format!("/api/v1/skills/{slug}/versions"), &[])?;
        let arr = v
            .get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .or_else(|| v.as_array().cloned())
            .unwrap_or_default();
        Ok(arr
            .into_iter()
            .filter_map(|x| {
                x.get("version")
                    .or_else(|| x.get("tag"))
                    .and_then(|s| s.as_str())
                    .map(|s| RemoteSkillVersion { version: s.into() })
            })
            .collect())
    }
    fn download(&self, slug: &str, version: Option<&str>) -> Result<Vec<u8>> {
        let ver = version.unwrap_or("latest");
        let resp = self
            .client
            .get(format!("{}/api/v1/download", self.base))
            .query(&[("slug", slug), ("version", ver)])
            .send()?;
        if resp.status().as_u16() == 429 {
            return Err(anyhow!("rate limited (429)"));
        }
        Ok(resp.bytes()?.to_vec())
    }
    fn pre_install_check(&self, slug: &str) -> Result<()> {
        let paths = ["scan", "verify"];
        for p in paths {
            let u = format!("{}/api/v1/skills/{}/{}", self.base, slug, p);
            if let Ok(r) = self.client.get(&u).send() {
                if r.status().is_success() {
                    let v: Value = r.json().unwrap_or(Value::Null);
                    let status = v.get("status").and_then(|x| x.as_str()).unwrap_or("");
                    if status.eq_ignore_ascii_case("malicious")
                        || status.eq_ignore_ascii_case("malware-blocked")
                    {
                        return Err(anyhow!("blocked by security scan: {status}"));
                    }
                }
            }
        }
        Ok(())
    }
}
