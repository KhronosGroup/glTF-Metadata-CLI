// SPDX-FileCopyrightText: 2020 2014-2020 The Khronos Group Inc.
//
// SPDX-License-Identifier: Apache-2.0

extern crate clap;
extern crate gltf;
extern crate serde;
extern crate serde_json;

use crate::io_helpers::{open_reader, open_writer, read_gltf, write_gltf, read_legacy_json, read_json};
use crate::json_models::extension::{Extension};
use crate::json_models::gltf::Gltf;
use crate::json_models::khr_xmp::KhrXmp;
use crate::managers::Manager;
use crate::managers::khr_xmp_manager::KhrXmpManager;
use clap::{App, Arg};
use gltf::Glb;
use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::process::exit;
use crate::json_models::khr_xmp_json_ld::KhrXmpJsonLd;
use crate::managers::khr_xmp_json_ld_manager::KhrXmpJsonLdManager;

mod io_helpers;
mod json_models;
mod managers;

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

pub enum PacketApplied {
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
// TODO: Need unit tests. A lot of unit tests.

/// Performs simple extension validation on a input path.
fn validate_file_as_extension(path: &Path, extension: &str) -> bool {
    match path.extension() {
        Some(ext) => ext.eq(OsStr::new(extension)),
        None => false,
    }
}

fn get_manager(g: Gltf, is_legacy: bool) -> Box<dyn Manager> {
    if is_legacy {
        Box::new(KhrXmpManager::new(g))
    } else {
        Box::new(KhrXmpJsonLdManager::new(g))
    }
}

fn list_gltf_metadata(path: &Path, is_legacy: bool) -> Result<(), String> {
    let reader = open_reader(path);
    let gltf = match reader {
        Ok(r) => read_gltf(r),
        Err(e) => Err(e),
    };

    match gltf {
        Ok(g) => {
            let manager = get_manager(g, is_legacy);
            manager.print_gltf()
        }
        Err(e) => Err(e.to_string()),
    }
}

fn list_glb_metadata(path: &Path, is_legacy: bool) -> Result<(), String> {
    let reader = match open_reader(path) {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };

    let glb = match Glb::from_reader(reader) {
        Ok(g) => g,
        Err(e) => return Err(e.to_string()),
    };

    match serde_json::from_slice(glb.json.as_ref()) {
        Ok(gltf) => {
            let manager = get_manager(gltf, is_legacy);
            manager.print_gltf()
        }
        Err(e) => Err(e.to_string()),
    }
}

fn log_if_verbose(verbose: bool, message: &str) {
    if verbose {
        println!("{}", message);
    }
}

// TODO: These update functions need to be moved to the managers.
//noinspection DuplicatedCode
fn update_gltf(
    input_path: &Path,
    output_path: &Path,
    metadata: &KhrXmpJsonLd,
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
        if let Some(xmp) = extensions.khr_xmp_json_ld.as_mut() {
            // TODO: Validation to make sure the input file isn't incorrect.
            xmp.packets = cloned.packets;
        } else {
            extensions.khr_xmp_json_ld = Some(KhrXmpJsonLd {
                packets: cloned.packets
            })
        }
    } else {
        gltf.extensions = Some(Extension {
            khr_xmp: None,
            khr_xmp_json_ld: Some(KhrXmpJsonLd {
                packets: cloned.packets,
            }),
            other_extensions: Default::default(),
        })
    }

    if let Some(extensions_used) = gltf.extensions_used.as_mut() {
        if !extensions_used.contains(&"KHR_xmp_json_ld".to_string()) {
            extensions_used.push("KHR_xmp_json_ld".to_string())
        }
    } else {
        gltf.extensions_used = Some(vec!["KHR_xmp_json_ld".to_string()])
    }

    let mut manager = KhrXmpJsonLdManager::new(gltf);

    log_if_verbose(is_verbose, "Clearing all applied packets.");
    manager.clear_applied_packets();
    log_if_verbose(is_verbose, "Setting new packets.");
    manager.set_applied_packets(apply_to);

    log_if_verbose(
        is_verbose,
        format!(
            "Opening & writing to output file at path {}",
            output_path.display()
        )
            .as_str(),
    );
    let output_writer = open_writer(output_path)?;
    write_gltf(output_writer, manager.get_gltf())?;

    Ok(())
}

//noinspection DuplicatedCode
fn update_glb(
    input_path: &Path,
    output_path: &Path,
    metadata: &KhrXmpJsonLd,
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
        if let Some(xmp) = extensions.khr_xmp_json_ld.as_mut() {
            // TODO: Validation to make sure the input file isn't incorrect.
            xmp.packets = cloned.packets;
        } else {
            extensions.khr_xmp_json_ld = Some(KhrXmpJsonLd {
                packets: cloned.packets
            })
        }
    } else {
        gltf.extensions = Some(Extension {
            khr_xmp: None,
            khr_xmp_json_ld: Some(KhrXmpJsonLd {
                packets: cloned.packets,
            }),
            other_extensions: Default::default(),
        })
    }

    if let Some(extensions_used) = gltf.extensions_used.as_mut() {
        if !extensions_used.contains(&"KHR_xmp_json_ld".to_string()) {
            extensions_used.push("KHR_xmp_json_ld".to_string())
        }
    } else {
        gltf.extensions_used = Some(vec!["KHR_xmp_json_ld".to_string()])
    }

    let mut manager = KhrXmpJsonLdManager::new(gltf);

    log_if_verbose(is_verbose, "Clearing all applied packets.");
    manager.clear_applied_packets();
    log_if_verbose(is_verbose, "Setting new packets.");
    manager.set_applied_packets(apply_to);

    let json_data = serde_json::to_string_pretty(manager.get_gltf())?;
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

// TODO: Probably can find a better way to handle updating using traits. I need to clean up this duplicate code.
//noinspection DuplicatedCode
fn update_gltf_legacy(
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
        } else {
            extensions.khr_xmp = Some(KhrXmp {
                context: cloned.context,
                packets: cloned.packets,
            })
        }
    } else {
        gltf.extensions = Some(Extension {
            khr_xmp: Some(KhrXmp {
                context: cloned.context,
                packets: cloned.packets,
            }),
            khr_xmp_json_ld: None,
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

    let mut manager = KhrXmpManager::new(gltf);

    log_if_verbose(is_verbose, "Clearing all applied packets.");
    manager.clear_applied_packets();
    log_if_verbose(is_verbose, "Setting new packets.");
    manager.set_applied_packets(apply_to);

    log_if_verbose(
        is_verbose,
        format!(
            "Opening & writing to output file at path {}",
            output_path.display()
        )
            .as_str(),
    );
    let output_writer = open_writer(output_path)?;
    write_gltf(output_writer, manager.get_gltf())?;

    Ok(())
}

//noinspection DuplicatedCode
fn update_glb_legacy(
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
        } else {
            extensions.khr_xmp = Some(KhrXmp {
                context: cloned.context,
                packets: cloned.packets,
            })
        }
    } else {
        gltf.extensions = Some(Extension {
            khr_xmp: Some(KhrXmp {
                context: cloned.context,
                packets: cloned.packets,
            }),
            khr_xmp_json_ld: None,
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

    let mut manager = KhrXmpManager::new(gltf);

    log_if_verbose(is_verbose, "Clearing all applied packets.");
    manager.clear_applied_packets();
    log_if_verbose(is_verbose, "Setting new packets.");
    manager.set_applied_packets(apply_to);

    let json_data = serde_json::to_string_pretty(manager.get_gltf())?;
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
                .help("Use raw JSON input file mode")
                // .required_unless("xmp")
                .required_unless("migrate")
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
            Arg::with_name("legacy")
                .long("legacy")
                .conflicts_with("migrate")
                .hidden(true)
                .help("Go through the legacy KHR_xmp flow. Testing only. Do not use.")
        )
        .arg(
            Arg::with_name("migrate")
                .short("m")
                .long("migrate")
                .help("Migrate KHR_xmp data to KHR_xmp_json_ld data")
        )
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

    // Check Legacy Mode
    let is_legacy = matches.is_present("legacy");

    // Check migration mode
    let migration = matches.is_present("migrate");

    if (migration) {
        clean_exit(ExitCode::Error, Some("Migration mode not fully implemented."))
    }

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
            InputType::Gltf => list_gltf_metadata(input_path, is_legacy),
            InputType::Glb => list_glb_metadata(input_path, is_legacy),
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
            // TODO: Need to move this to the managers.
            if is_legacy {
                // KHR_xmp
                let metadata_path = Path::new(p.as_str());
                let metadata = match open_reader(metadata_path) {
                    Ok(file) => read_legacy_json(file),
                    Err(e) => Err(e),
                };
                match metadata {
                    Ok(m) => match input_type {
                        InputType::Gltf => {
                            match update_gltf_legacy(input_path, output_path, &m, apply_to, verbose) {
                                Err(e) => return exit_on_error(e),
                                _ => (),
                            }
                        }
                        InputType::Glb => {
                            match update_glb_legacy(input_path, output_path, &m, apply_to, verbose) {
                                Err(e) => return exit_on_error(e),
                                _ => (),
                            }
                        }
                    },
                    Err(e) => return exit_on_error(e),
                }
            } else {
                // KHR_xmp_json_ld
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
                        }
                    },
                    Err(e) => return exit_on_error(e),
                }
            };
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
