// SPDX-FileCopyrightText: 2020 2014-2020 The Khronos Group Inc.
//
// SPDX-License-Identifier: Apache-2.0

use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct KhrXmpJsonLdPacket {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct KhrXmpJsonLdPacketList {
    #[serde(rename = "@context")]
    pub context: Value,

    #[serde(flatten)]
    pub metadata: HashMap<String, Value>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KhrXmpJsonLd {
    pub packets: Vec<Value>,
}

