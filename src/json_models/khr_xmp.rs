// SPDX-FileCopyrightText: 2020 2014-2020 The Khronos Group Inc.
//
// SPDX-License-Identifier: Apache-2.0

use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct KhrXmpPacket {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KhrXmp {
    #[serde(rename = "@context")]
    pub context: Value,
    pub packets: Vec<Value>,
}
