// SPDX-FileCopyrightText: 2020 2014-2020 The Khronos Group Inc.
//
// SPDX-License-Identifier: Apache-2.0

use crate::json_models::gltf::Gltf;
use crate::PacketApplied;

pub mod khr_xmp_manager;

pub trait Manager {
    fn new(g: Gltf) -> Self;

    fn get_gltf(&self) -> &Gltf;

    fn print_gltf(&self) -> Result<(), String>;

    fn clear_applied_packets(&mut self);

    fn set_applied_packets(&mut self, apply_to: Vec<PacketApplied>);
}
