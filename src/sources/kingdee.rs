#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KingdeeSource {
    pub id: i64,
    pub version: String,
}
