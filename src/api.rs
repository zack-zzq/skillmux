use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

include!(concat!(env!("OUT_DIR"), "/generated_token.rs"));

#[derive(Clone)]
pub struct ApiClient {
    base: String,
    token: String,
    client: Client,
}

impl ApiClient {
    pub fn new(base: String, timeout: u64, token: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout))
            .build()?;
        Ok(Self {
            base: base.trim_end_matches('/').to_string(),
            token,
            client,
        })
    }
    pub fn default_token() -> String {
        TOKEN_DATA.iter().map(|b| (b ^ TOKEN_KEY) as char).collect()
    }
    pub fn search(&self, keyword: Option<String>, page: u32, page_size: u32) -> Result<Value> {
        let mut req = self
            .client
            .get(format!("{}/skills/list", self.base))
            .query(&[("page", page), ("pageSize", page_size)]);
        if let Some(k) = keyword.as_ref() {
            req = req.query(&[("keyword", k)]);
        }
        Ok(req.headers(self.headers()).send()?.json()?)
    }
    pub fn download(&self, id: i64, version: &str) -> Result<Vec<u8>> {
        Ok(self
            .client
            .get(format!("{}/skills/download", self.base))
            .query(&[
                ("id", id.to_string()),
                ("version", version.to_string()),
                ("token", self.token.clone()),
            ])
            .headers(self.headers())
            .send()?
            .bytes()?
            .to_vec())
    }
    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        h.insert("accept", "application/json".parse().unwrap());
        h.insert("content-type", "application/json".parse().unwrap());
        h.insert(
            "user-agent",
            format!("skillhub-cli/{}", env!("CARGO_PKG_VERSION"))
                .parse()
                .unwrap(),
        );
        if let Ok(v) = self.token.parse() {
            h.insert("token", v);
        }
        h
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillInfo {
    pub id: i64,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "currentVersion")]
    pub current_version: Option<String>,
    pub version: Option<String>,
}

pub fn extract_list(v: &Value) -> Result<Vec<SkillInfo>> {
    let list = v
        .get("data")
        .and_then(|d| d.get("list"))
        .ok_or_else(|| anyhow!("missing data.list"))?;
    Ok(serde_json::from_value(list.clone())?)
}
