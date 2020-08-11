use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::json_models::extension::PacketExtension;

#[derive(Serialize, Deserialize)]
pub struct Asset {
    pub version: String,
    pub extensions: Option<PacketExtension>,

    #[serde(flatten)]
    pub other_fields: HashMap<String, Value>,
}
