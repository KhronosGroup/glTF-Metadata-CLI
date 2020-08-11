use crate::json_models::asset::Asset;
use crate::json_models::extension::{Extension, ExtensionsOnly};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Struct used to represent the entire glTF JSON file.
#[derive(Serialize, Deserialize)]
pub struct Gltf {
    pub asset: Asset,

    #[serde(rename = "extensionsUsed", skip_serializing_if = "Option::is_none")]
    pub extensions_used: Option<Vec<String>>,

    // TODO: Consider removing accessors irrelevant to our needs. Lots here that can't accept a KHR_XMP extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessors: Option<Vec<ExtensionsOnly>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<Vec<ExtensionsOnly>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffers: Option<Vec<ExtensionsOnly>>,

    #[serde(rename = "bufferViews")]
    pub buffer_views: Option<Vec<ExtensionsOnly>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cameras: Option<Vec<ExtensionsOnly>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<ExtensionsOnly>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub materials: Option<Vec<ExtensionsOnly>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meshes: Option<Vec<ExtensionsOnly>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<ExtensionsOnly>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub samplers: Option<Vec<ExtensionsOnly>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenes: Option<Vec<ExtensionsOnly>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skins: Option<Vec<ExtensionsOnly>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub textures: Option<Vec<ExtensionsOnly>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extension>,

    #[serde(flatten)]
    pub other_fields: HashMap<String, Value>,
}
