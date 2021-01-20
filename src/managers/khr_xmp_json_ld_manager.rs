use crate::json_models::gltf::Gltf;
use crate::managers::Manager;
use crate::{PacketApplied, NO_METADATA_FOUND_ERROR};
use crate::json_models::extension::{ExtensionsOnly, PacketExtension};
use crate::json_models::khr_xmp_json_ld::KhrXmpJsonLdPacket;

pub struct KhrXmpJsonLdManager {
    gltf: Gltf,
}

impl Manager for KhrXmpJsonLdManager {
    fn new(g: Gltf) -> Self {
        KhrXmpJsonLdManager { gltf: g }
    }

    fn get_gltf(&self) -> &Gltf {
        &self.gltf
    }

    fn print_gltf(&self) -> Result<(), String> {
        match &self.gltf.extensions {
            Some(extension) => match &extension.khr_xmp_json_ld {
                // TODO: This is currently duplicated in both managers. It can probably be lifted out.
                Some(xmp) => {
                    let mut applied_packets: Vec<PacketApplied> = vec![];
                    let empty_vec: Vec<ExtensionsOnly> = Vec::new();

                    match get_packet_value(&self.gltf.asset.extensions) {
                        Some(index) => applied_packets.push(PacketApplied::Asset(index)),
                        None => (),
                    }

                    // TODO: Need to clean this up. Lots of repeat code here.
                    for animation in self.gltf.animations.as_ref().unwrap_or(&empty_vec) {
                        match get_packet_value(&animation.extensions) {
                            Some(index) => applied_packets.push(PacketApplied::Animations(index)),
                            None => (),
                        }
                    }

                    for image in self.gltf.images.as_ref().unwrap_or(&empty_vec) {
                        match get_packet_value(&image.extensions) {
                            Some(index) => applied_packets.push(PacketApplied::Images(index)),
                            None => (),
                        }
                    }

                    for material in self.gltf.materials.as_ref().unwrap_or(&empty_vec) {
                        match get_packet_value(&material.extensions) {
                            Some(index) => applied_packets.push(PacketApplied::Materials(index)),
                            None => (),
                        }
                    }

                    for mesh in self.gltf.meshes.as_ref().unwrap_or(&empty_vec) {
                        match get_packet_value(&mesh.extensions) {
                            Some(index) => applied_packets.push(PacketApplied::Meshes(index)),
                            None => (),
                        }
                    }

                    for node in self.gltf.nodes.as_ref().unwrap_or(&empty_vec) {
                        match get_packet_value(&node.extensions) {
                            Some(index) => applied_packets.push(PacketApplied::Nodes(index)),
                            None => (),
                        }
                    }

                    for scene in self.gltf.scenes.as_ref().unwrap_or(&empty_vec) {
                        match get_packet_value(&scene.extensions) {
                            Some(index) => applied_packets.push(PacketApplied::Scenes(index)),
                            None => (),
                        }
                    }

                    println!("KHR_xmp_json_ld extension value:");
                    println!("{}", serde_json::to_string_pretty(&xmp).unwrap());

                    // TODO: I want to also include the name, if available, to make it easier to
                    //   figure out which packet corresponds exactly to which image/mesh/etc.
                    println!("\nPackets applied at:");
                    for packet in applied_packets {
                        match packet {
                            PacketApplied::Asset(index) => println!("\tAssets: {}", index),
                            PacketApplied::Animations(index) => println!("\tAnimations: {}", index),
                            PacketApplied::Images(index) => println!("\tImages: {}", index),
                            PacketApplied::Materials(index) => println!("\tMaterials: {}", index),
                            PacketApplied::Meshes(index) => println!("\tMeshes: {}", index),
                            PacketApplied::Nodes(index) => println!("\tNodes: {}", index),
                            PacketApplied::Scenes(index) => println!("\tScenes: {}", index),
                        }
                    }

                    Ok(())
                },
                None => Err(NO_METADATA_FOUND_ERROR.to_string()),
            },
            None => Err(NO_METADATA_FOUND_ERROR.to_string()),
        }
    }

    fn clear_applied_packets(&mut self) {
        // There is maybe a more elegant way to do this, but brute force it for now.
        if let Some(mut e) = self.gltf.asset.extensions.as_mut() {
            e.khr_xmp_json_ld = None;
        }

        if *&self.gltf.animations.is_some() {
            for animation in self.gltf.animations.as_mut().unwrap() {
                if let Some(mut e) = animation.extensions.as_mut() {
                    e.khr_xmp_json_ld = None;
                }
            }
        }

        if *&self.gltf.images.is_some() {
            for image in self.gltf.images.as_mut().unwrap() {
                if let Some(mut e) = image.extensions.as_mut() {
                    e.khr_xmp_json_ld = None;
                }
            }
        }

        if *&self.gltf.materials.is_some() {
            for material in self.gltf.materials.as_mut().unwrap() {
                if let Some(mut e) = material.extensions.as_mut() {
                    e.khr_xmp_json_ld = None;
                }
            }
        }

        if *&self.gltf.meshes.is_some() {
            for mesh in self.gltf.meshes.as_mut().unwrap() {
                if let Some(mut e) = mesh.extensions.as_mut() {
                    e.khr_xmp_json_ld = None;
                }
            }
        }

        if *&self.gltf.nodes.is_some() {
            for node in self.gltf.nodes.as_mut().unwrap() {
                if let Some(mut e) = node.extensions.as_mut() {
                    e.khr_xmp_json_ld = None;
                }
            }
        }

        if *&self.gltf.scenes.is_some() {
            for scene in self.gltf.scenes.as_mut().unwrap() {
                if let Some(mut e) = scene.extensions.as_mut() {
                    e.khr_xmp_json_ld = None;
                }
            }
        }
    }

    fn set_applied_packets(&mut self, apply_to: Vec<PacketApplied>) {
        // Right now we just apply to _all_ of a category.
        for packet in apply_to {
            match packet {
                PacketApplied::Asset(i) => {
                    if let Some(e) = self.gltf.asset.extensions.as_mut() {
                        e.khr_xmp_json_ld = Some(KhrXmpJsonLdPacket { packet: Some(i) });
                    } else {
                        self.gltf.asset.extensions = Some(PacketExtension {
                            khr_xmp: None,
                            khr_xmp_json_ld: Some(KhrXmpJsonLdPacket { packet: Some(i) }),
                            other_extensions: Default::default(),
                        });
                    };
                }
                PacketApplied::Animations(i) => {
                    for animation in self.gltf.animations.as_mut().unwrap() {
                        if let Some(e) = animation.extensions.as_mut() {
                            e.khr_xmp_json_ld = Some(KhrXmpJsonLdPacket { packet: Some(i) });
                        } else {
                            animation.extensions = Some(PacketExtension {
                                khr_xmp: None,
                                khr_xmp_json_ld: Some(KhrXmpJsonLdPacket { packet: Some(i) }),
                                other_extensions: Default::default(),
                            });
                        };
                    }
                }
                PacketApplied::Images(i) => {
                    for image in self.gltf.images.as_mut().unwrap() {
                        if let Some(e) = image.extensions.as_mut() {
                            e.khr_xmp_json_ld = Some(KhrXmpJsonLdPacket { packet: Some(i) });
                        } else {
                            image.extensions = Some(PacketExtension {
                                khr_xmp: None,
                                khr_xmp_json_ld: Some(KhrXmpJsonLdPacket { packet: Some(i) }),
                                other_extensions: Default::default(),
                            });
                        };
                    }
                }
                PacketApplied::Materials(i) => {
                    for material in self.gltf.materials.as_mut().unwrap() {
                        if let Some(e) = material.extensions.as_mut() {
                            e.khr_xmp_json_ld = Some(KhrXmpJsonLdPacket { packet: Some(i) });
                        } else {
                            material.extensions = Some(PacketExtension {
                                khr_xmp: None,
                                khr_xmp_json_ld: Some(KhrXmpJsonLdPacket { packet: Some(i) }),
                                other_extensions: Default::default(),
                            });
                        };
                    }
                }
                PacketApplied::Meshes(i) => {
                    for mesh in self.gltf.meshes.as_mut().unwrap() {
                        if let Some(e) = mesh.extensions.as_mut() {
                            e.khr_xmp_json_ld = Some(KhrXmpJsonLdPacket { packet: Some(i) });
                        } else {
                            mesh.extensions = Some(PacketExtension {
                                khr_xmp: None,
                                khr_xmp_json_ld: Some(KhrXmpJsonLdPacket { packet: Some(i) }),
                                other_extensions: Default::default(),
                            });
                        };
                    }
                }
                PacketApplied::Nodes(i) => {
                    for node in self.gltf.nodes.as_mut().unwrap() {
                        if let Some(e) = node.extensions.as_mut() {
                            e.khr_xmp_json_ld = Some(KhrXmpJsonLdPacket { packet: Some(i) });
                        } else {
                            node.extensions = Some(PacketExtension {
                                khr_xmp: None,
                                khr_xmp_json_ld: Some(KhrXmpJsonLdPacket { packet: Some(i) }),
                                other_extensions: Default::default(),
                            });
                        };
                    }
                }
                PacketApplied::Scenes(i) => {
                    for scene in self.gltf.scenes.as_mut().unwrap() {
                        if let Some(e) = scene.extensions.as_mut() {
                            e.khr_xmp_json_ld = Some(KhrXmpJsonLdPacket { packet: Some(i) });
                        } else {
                            scene.extensions = Some(PacketExtension {
                                khr_xmp: None,
                                khr_xmp_json_ld: Some(KhrXmpJsonLdPacket { packet: Some(i) }),
                                other_extensions: Default::default(),
                            });
                        };
                    }
                }
            }
        }
    }
}

fn get_packet_value(extension: &Option<PacketExtension>) -> Option<u64> {
    match extension {
        None => None,
        Some(ex) => match &ex.khr_xmp_json_ld {
            None => None,
            Some(xmp) => match xmp.packet {
                None => None,
                Some(index) => Some(index),
            },
        },
    }
}
