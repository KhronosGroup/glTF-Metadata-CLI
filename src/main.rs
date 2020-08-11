extern crate clap;
extern crate gltf;
extern crate serde;
extern crate serde_json;

use crate::io_helpers::{open_reader, open_writer, read_gltf, read_json, write_gltf};
use crate::json_models::extension::{Extension, PacketExtension};
use crate::json_models::gltf::Gltf;
use crate::json_models::khr_xmp::{KhrXmp, KhrXmpPacket};
use clap::{App, Arg};
use gltf::Glb;
use std::borrow::{Borrow, Cow};
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::process::exit;

mod io_helpers;
mod json_models;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");
const USAGE: &str = "gltfxmp [FLAGS] -j <JSON_FILE> -i <IN_FILE> -o <OUT_FILE>";

const NO_METADATA_FOUND_ERROR: &str = "No metadata found.";

enum InputType {
    Gltf,
    Glb,
}

enum MetadataInputMode {
    JSON(String),
    XMP(String),
    Manual,
}

enum PacketApplied {
    Asset(u64),
    Animations(u64),
    Images(u64),
    Materials(u64),
    Meshes(u64),
    Nodes(u64),
    Scenes(u64),
}

#[derive(PartialEq)]
enum ExitCode {
    Normal = 0,
    Error = 1,
    Warn = 1000,
}

// TODO: Further reduce the number of unwraps to increase safety.
// TODO: Move a lot of this code into separate modules.
// TODO: Need unit tests. A lot of unit tests.

/// Performs simple extension validation on a input path.
fn validate_file_as_extension(path: &Path, extension: &str) -> bool {
    match path.extension() {
        Some(ext) => ext.eq(OsStr::new(extension)),
        None => false,
    }
}

fn list_gltf_metadata(path: &Path) -> Result<(), String> {
    let reader = open_reader(path);
    let gltf = match reader {
        Ok(r) => read_gltf(r),
        Err(e) => Err(e),
    };

    match gltf {
        Ok(g) => print_gltf(g),
        Err(e) => Err(e.to_string()),
    }
}

fn get_packet_value(extension: Option<PacketExtension>) -> Option<u64> {
    match extension {
        None => None,
        Some(ex) => match ex.khr_xmp {
            None => None,
            Some(xmp) => match xmp.packet {
                None => None,
                Some(index) => Some(index),
            },
        },
    }
}

fn print_gltf(gltf: Gltf) -> Result<(), String> {
    match gltf.extensions {
        Some(extension) => match extension.khr_xmp {
            Some(xmp) => {
                let mut applied_packets: Vec<PacketApplied> = vec![];

                match get_packet_value(gltf.asset.extensions) {
                    Some(index) => applied_packets.push(PacketApplied::Asset(index)),
                    None => (),
                }

                // TODO: Need to clean this up. Lots of repeat code here.
                for animation in gltf.animations.unwrap_or_default() {
                    match get_packet_value(animation.extensions) {
                        Some(index) => applied_packets.push(PacketApplied::Animations(index)),
                        None => (),
                    }
                }

                for image in gltf.images.unwrap_or_default() {
                    match get_packet_value(image.extensions) {
                        Some(index) => applied_packets.push(PacketApplied::Images(index)),
                        None => (),
                    }
                }

                for material in gltf.materials.unwrap_or_default() {
                    match get_packet_value(material.extensions) {
                        Some(index) => applied_packets.push(PacketApplied::Materials(index)),
                        None => (),
                    }
                }

                for mesh in gltf.meshes.unwrap_or_default() {
                    match get_packet_value(mesh.extensions) {
                        Some(index) => applied_packets.push(PacketApplied::Meshes(index)),
                        None => (),
                    }
                }

                for node in gltf.nodes.unwrap_or_default() {
                    match get_packet_value(node.extensions) {
                        Some(index) => applied_packets.push(PacketApplied::Nodes(index)),
                        None => (),
                    }
                }

                for scene in gltf.scenes.unwrap_or_default() {
                    match get_packet_value(scene.extensions) {
                        Some(index) => applied_packets.push(PacketApplied::Scenes(index)),
                        None => (),
                    }
                }

                println!("KHR_xmp extension value:");
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
            }
            None => Err(NO_METADATA_FOUND_ERROR.to_string()),
        },
        None => Err(NO_METADATA_FOUND_ERROR.to_string()),
    }
}

fn list_glb_metadata(path: &Path) -> Result<(), String> {
    let reader = match open_reader(path) {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };

    let glb = match Glb::from_reader(reader) {
        Ok(g) => g,
        Err(e) => return Err(e.to_string()),
    };

    match serde_json::from_slice(glb.json.as_ref()) {
        Ok(gltf) => print_gltf(gltf),
        Err(e) => Err(e.to_string()),
    }
}

fn log_if_verbose(verbose: bool, message: &str) {
    if verbose {
        println!("{}", message);
    }
}

fn clear_applied_packets(gltf: &mut Gltf) {
    // There is maybe a more elegant way to do this, but brute force it for now.
    if let Some(mut e) = gltf.asset.extensions.as_mut() {
        e.khr_xmp = None;
    }

    if gltf.animations.is_some() {
        for animation in gltf.animations.as_mut().unwrap() {
            if let Some(mut e) = animation.extensions.as_mut() {
                e.khr_xmp = None;
            }
        }
    }

    if gltf.images.is_some() {
        for image in gltf.images.as_mut().unwrap() {
            if let Some(mut e) = image.extensions.as_mut() {
                e.khr_xmp = None;
            }
        }
    }

    if gltf.materials.is_some() {
        for material in gltf.materials.as_mut().unwrap() {
            if let Some(mut e) = material.extensions.as_mut() {
                e.khr_xmp = None;
            }
        }
    }

    if gltf.meshes.is_some() {
        for mesh in gltf.meshes.as_mut().unwrap() {
            if let Some(mut e) = mesh.extensions.as_mut() {
                e.khr_xmp = None;
            }
        }
    }

    if gltf.nodes.is_some() {
        for node in gltf.nodes.as_mut().unwrap() {
            if let Some(mut e) = node.extensions.as_mut() {
                e.khr_xmp = None;
            }
        }
    }

    if gltf.scenes.is_some() {
        for scene in gltf.scenes.as_mut().unwrap() {
            if let Some(mut e) = scene.extensions.as_mut() {
                e.khr_xmp = None;
            }
        }
    }
}

//noinspection DuplicatedCode There's a lot of repeat code here because dealing with the lifetimes of the serde object is difficult due to our use of flatten.
fn set_applied_packets(gltf: &mut Gltf, apply_to: Vec<PacketApplied>) {
    // Right now we just apply to _all_ of a category.
    for packet in apply_to {
        match packet {
            PacketApplied::Asset(i) => {
                if let Some(e) = gltf.asset.extensions.as_mut() {
                    e.khr_xmp = Some(KhrXmpPacket { packet: Some(i) });
                } else {
                    gltf.asset.extensions = Some(PacketExtension {
                        khr_xmp: Some(KhrXmpPacket { packet: Some(i) }),
                        other_extensions: Default::default(),
                    });
                };
            }
            PacketApplied::Animations(i) => {
                for animation in gltf.animations.as_mut().unwrap() {
                    if let Some(e) = animation.extensions.as_mut() {
                        e.khr_xmp = Some(KhrXmpPacket { packet: Some(i) });
                    } else {
                        animation.extensions = Some(PacketExtension {
                            khr_xmp: Some(KhrXmpPacket { packet: Some(i) }),
                            other_extensions: Default::default(),
                        });
                    };
                }
            }
            PacketApplied::Images(i) => {
                for image in gltf.images.as_mut().unwrap() {
                    if let Some(e) = image.extensions.as_mut() {
                        e.khr_xmp = Some(KhrXmpPacket { packet: Some(i) });
                    } else {
                        image.extensions = Some(PacketExtension {
                            khr_xmp: Some(KhrXmpPacket { packet: Some(i) }),
                            other_extensions: Default::default(),
                        });
                    };
                }
            }
            PacketApplied::Materials(i) => {
                for material in gltf.materials.as_mut().unwrap() {
                    if let Some(e) = material.extensions.as_mut() {
                        e.khr_xmp = Some(KhrXmpPacket { packet: Some(i) });
                    } else {
                        material.extensions = Some(PacketExtension {
                            khr_xmp: Some(KhrXmpPacket { packet: Some(i) }),
                            other_extensions: Default::default(),
                        });
                    };
                }
            }
            PacketApplied::Meshes(i) => {
                for mesh in gltf.meshes.as_mut().unwrap() {
                    if let Some(e) = mesh.extensions.as_mut() {
                        e.khr_xmp = Some(KhrXmpPacket { packet: Some(i) });
                    } else {
                        mesh.extensions = Some(PacketExtension {
                            khr_xmp: Some(KhrXmpPacket { packet: Some(i) }),
                            other_extensions: Default::default(),
                        });
                    };
                }
            }
            PacketApplied::Nodes(i) => {
                for node in gltf.nodes.as_mut().unwrap() {
                    if let Some(e) = node.extensions.as_mut() {
                        e.khr_xmp = Some(KhrXmpPacket { packet: Some(i) });
                    } else {
                        node.extensions = Some(PacketExtension {
                            khr_xmp: Some(KhrXmpPacket { packet: Some(i) }),
                            other_extensions: Default::default(),
                        });
                    };
                }
            }
            PacketApplied::Scenes(i) => {
                for scene in gltf.scenes.as_mut().unwrap() {
                    if let Some(e) = scene.extensions.as_mut() {
                        e.khr_xmp = Some(KhrXmpPacket { packet: Some(i) });
                    } else {
                        scene.extensions = Some(PacketExtension {
                            khr_xmp: Some(KhrXmpPacket { packet: Some(i) }),
                            other_extensions: Default::default(),
                        });
                    };
                }
            }
        }
    }
}

// TODO: Probably can find a better way to handle updating using traits. I need to clean up this duplicate code.
//noinspection DuplicatedCode
fn update_gltf(
    input_path: &Path,
    output_path: &Path,
    metadata: &KhrXmp,
    apply_to: Vec<PacketApplied>,
    is_verbose: bool,
) -> Result<(), Box<dyn Error>> {
    log_if_verbose(
        is_verbose,
        format!(
            "Opening & reading input file at path {}",
            input_path.display()
        )
        .as_str(),
    );
    let input_reader = open_reader(input_path)?;
    let mut gltf = read_gltf(input_reader)?;

    let cloned = metadata.clone();
    if let Some(extensions) = gltf.extensions.as_mut() {
        if let Some(xmp) = extensions.khr_xmp.as_mut() {
            // TODO: Validation to make sure the input file isn't incorrect.
            xmp.context = cloned.context;
            xmp.packets = cloned.packets;
        }
    } else {
        gltf.extensions = Some(Extension {
            khr_xmp: Some(KhrXmp {
                context: cloned.context,
                packets: cloned.packets,
            }),
            other_extensions: Default::default(),
        })
    }

    if let Some(extensions_used) = gltf.extensions_used.as_mut() {
        if !extensions_used.contains(&"KHR_xmp".to_string()) {
            extensions_used.push("KHR_xmp".to_string())
        }
    } else {
        gltf.extensions_used = Some(vec!["KHR_xmp".to_string()])
    }

    log_if_verbose(is_verbose, "Clearing all applied packets.");
    clear_applied_packets(&mut gltf);
    log_if_verbose(is_verbose, "Setting new packets.");
    set_applied_packets(&mut gltf, apply_to);

    log_if_verbose(
        is_verbose,
        format!(
            "Opening & writing to output file at path {}",
            output_path.display()
        )
        .as_str(),
    );
    let output_writer = open_writer(output_path)?;
    write_gltf(output_writer, &gltf)?;

    Ok(())
}

//noinspection DuplicatedCode
fn update_glb(
    input_path: &Path,
    output_path: &Path,
    metadata: &KhrXmp,
    apply_to: Vec<PacketApplied>,
    is_verbose: bool,
) -> Result<(), Box<dyn Error>> {
    log_if_verbose(
        is_verbose,
        format!(
            "Opening & reading input file at path {}",
            input_path.display()
        )
            .as_str(),
    );
    let input_reader = open_reader(input_path)?;
    let glb = match Glb::from_reader(input_reader) {
        Ok(g) => g,
        Err(e) => return Err(e.into()),
    };

    let mut gltf: Gltf = serde_json::from_slice(glb.json.as_ref())?;

    let cloned = metadata.clone();

    if let Some(extensions) = gltf.extensions.as_mut() {
        if let Some(xmp) = extensions.khr_xmp.as_mut() {
            // TODO: Validation to make sure the input file isn't incorrect.
            xmp.context = cloned.context;
            xmp.packets = cloned.packets;
        }
    } else {
        gltf.extensions = Some(Extension {
            khr_xmp: Some(KhrXmp {
                context: cloned.context,
                packets: cloned.packets,
            }),
            other_extensions: Default::default(),
        })
    }

    if let Some(extensions_used) = gltf.extensions_used.as_mut() {
        if !extensions_used.contains(&"KHR_xmp".to_string()) {
            extensions_used.push("KHR_xmp".to_string())
        }
    } else {
        gltf.extensions_used = Some(vec!["KHR_xmp".to_string()])
    }

    log_if_verbose(is_verbose, "Clearing all applied packets.");
    clear_applied_packets(&mut gltf);
    log_if_verbose(is_verbose, "Setting new packets.");
    set_applied_packets(&mut gltf, apply_to);

    let json_data = serde_json::to_string_pretty(&gltf)?;
    let json_offset = align_to_multiple_of_four(glb.json.len() as u32);

    let new_bin = glb.bin.unwrap_or_default().clone();
    let new_glb = gltf::binary::Glb {
        header: gltf::binary::Header {
            magic: b"glTF".clone(),
            version: 2,
            length: json_offset + new_bin.len() as u32,
        },
        json: Cow::Owned(json_data.into_bytes()),
        bin: Some(new_bin),
    };

    let writer = std::fs::File::create(output_path)?;
    new_glb.to_writer(writer)?;

    Ok(())
}

fn align_to_multiple_of_four(n: u32) -> u32 {
    return (n + 3) & !3;
}

/// Performs a graceful exit with the specified `ExitCode` and an optional message.
fn clean_exit(code: ExitCode, message: Option<&str>) {
    if message.is_some() {
        if code == ExitCode::Error {
            eprintln!("{}", message.unwrap_or("Unknown Error."));
        } else {
            println!("{}", message.unwrap_or("Unknown Error."));
        }
    }

    exit(code as i32);
}

/// Performs a graceful exit, printing the error message contained in the error.
fn exit_on_error(e: Box<dyn Error>) {
    clean_exit(ExitCode::Error, Some(e.to_string().as_str()))
}

fn main() {
    let matches = App::new(NAME)
        .version(VERSION)
        .about(ABOUT)
        .usage(USAGE)
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("IN_FILE")
                .help("Input file path")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUT_FILE")
                .help("Output file path")
                .required_unless("list")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .help("Lists pretty-printed Metadata from input glTF file."),
        )
        .arg(
            Arg::with_name("json")
                .short("j")
                .long("json")
                .value_name("JSON_FILE")
                .help("Use JSON input file mode")
                // .required_unless("xmp")
                .required_unless("list"), // .conflicts_with("xmp"),
        )
        // .arg(
        //     Arg::with_name("xmp")
        //         .short("x")
        //         .long("xmp")
        //         .help("Use XMP input file mode")
        //         .required_unless("json")
        //         .required_unless("list")
        //         .conflicts_with("json"),
        // )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .takes_value(false)
                .help("Verbose logging"),
        )
        .arg(
            Arg::with_name("allow_overwrite")
                .long("allow-overwrite")
                .help("Allows output file overwriting. Use at your own risk!"),
        )
        .get_matches();

    // Check verbosity
    let verbose = matches.is_present("verbose");

    // TODO: Fully implement apply_to logic.
    let apply_to = vec![PacketApplied::Asset(0)];

    // Read input file path.
    let input_path = Path::new(matches.value_of("input").unwrap());

    if !input_path.exists() && input_path.is_file() {
        let message = format!(
            "The input file provided, {}, does not exist or is inaccessible.",
            input_path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
        );
        clean_exit(ExitCode::Error, Some(message.as_str()))
    }

    let input_type = match (
        validate_file_as_extension(input_path, "gltf"),
        validate_file_as_extension(input_path, "glb"),
    ) {
        (true, _) => InputType::Gltf,
        (_, true) => InputType::Glb,
        (_, _) => {
            return clean_exit(
                ExitCode::Error,
                Some("Input file must be either a gltf or glb!"),
            );
        }
    };

    if matches.is_present("list") {
        let result = match input_type {
            InputType::Gltf => list_gltf_metadata(input_path),
            InputType::Glb => list_glb_metadata(input_path),
        };

        return match result {
            Err(e) => clean_exit(ExitCode::Error, Some(e.as_str())),
            _ => clean_exit(ExitCode::Normal, None),
        };
    }

    let output_path = Path::new(matches.value_of("output").unwrap());

    // TODO: Add a flag to allow overwriting files.
    if output_path.exists() && !matches.is_present("allow_overwrite") {
        let message = format!(
            "The output path provided {}, already exists. This tool does not overwrite files by default. Use the --allow-overwrite flag to allow overwriting.",
            output_path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
        );
        clean_exit(ExitCode::Error, Some(message.as_str()))
    }

    let mode = match (matches.is_present("json"), matches.is_present("xmp")) {
        (true, _) => {
            MetadataInputMode::JSON(matches.value_of("json").unwrap_or_default().to_owned())
        }
        (_, true) => MetadataInputMode::XMP(matches.value_of("xmp").unwrap_or_default().to_owned()),
        (_, _) => {
            // We need to fatally die here. Something is probably wrong with the Clap config.
            panic!("FATAL: No mode set, but validation already performed. Check Clap config!")
        }
    };
    match mode {
        MetadataInputMode::JSON(p) => {
            let metadata_path = Path::new(p.as_str());
            let metadata = match open_reader(metadata_path) {
                Ok(file) => read_json(file),
                Err(e) => Err(e),
            };
            match metadata {
                Ok(m) => match input_type {
                    InputType::Gltf => {
                        match update_gltf(input_path, output_path, &m, apply_to, verbose) {
                            Err(e) => return exit_on_error(e),
                            _ => (),
                        }
                    }
                    InputType::Glb => {
                        match update_glb(input_path, output_path, &m, apply_to, verbose) {
                            Err(e) => return exit_on_error(e),
                            _ => (),
                        }
                    },
                },
                Err(e) => return exit_on_error(e),
            }
        }
        MetadataInputMode::XMP(_path) => {
            // TODO: Add XMP file input support.
            return clean_exit(ExitCode::Error, Some("XMP input is not yet implemented."));
        }
        MetadataInputMode::Manual => {
            // TODO: Add manual input support.
            return clean_exit(
                ExitCode::Error,
                Some("Manual input is not yet implemented."),
            );
        }
    }
}
