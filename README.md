# glTF Metadata CLI Tool

CLI tool for managing metadata within glTF files using the KHR_XMP extension.

## NOTICE

**Binaries are not yet available, but will be coming very soon!** In the meantime please follow the section below if you'd like to try out the alpha release.

# How to use

Currently, binaries are not available. This will be rectified soon. In order to build a copy for yourself, you will need to install and configure [Rust for your platform](https://www.rust-lang.org/tools/install).

Once Rust is installed, follow these steps:

 1. Pull down the latest copy of this code from the `v1.0.0-alpha01` tag: `git checkout v1.0.0-alpha01`
 2. Run `cargo build`. This will install and build dependencies.
 3. Run `cargo run -- <YOUR ARGUMENTS HERE>`. Some examples below:
 
To list out the metadata found in the `Box.gltf` file, use:

```shell script
cargo run -- -i examples/Box.gltf --list
```

To create a version of the Boombox sample with metadata use:

```shell script
cargo run -- -i glTF-Sample-Models/2.0/BoomBox/glTF/BoomBox.gltf -o glTF-Sample-Models/2.0/BoomBox/glTF/BoomBox_metadata.gltf -j examples/sample.khr_xmp.json
```

You can also read and write from `.glb` files as well. To create a version of the DamagedHelmet binary glTF sample with metadata, use:

```shell script
cargo run -- -i ./glTF-Sample-Models/2.0/DamagedHelmet/glTF-Binary/DamagedHelmet.glb -o DamagedHelmet_metadata.glb -j examples/sample.khr_xmp.json
```

# Available Arguments

| Flag | Value | Description | Required? | Version Added |
| --- | --- | --- | --- | --- |
| `-i`, `--input` | Path | Input file path | Yes | 1.0.0-alpha01 |
| `-o`, `--output` | Path | Output file path | Yes, unless `--list` flag present. | 1.0.0-alpha01 |
| `-l`, `--list` | None | Lists the metadata out to the console. | No | 1.0.0-alpha01 |
| `-j`, `--json` | Path | JSON file path including KHR_xmp metadata | No | 1.0.0-alpha01 |
| `--allow-overwrite` | None | Allow overwriting the output file. | No | 1.0.0-alpha01 |
| `-v`, `--verbose` | None | Enable verbose logging output. | No | 1.0.0-alpha01 | 

# Future milestones

This section is formatted as "PRIORITY: Milestone" to give an idea of how important the milestone is to final 1.0.0 release.

 - CRITICAL: Migration flag to migrate from *KHR_xmp* to *KHR_xmp_json_ld*. 
 - CRITICAL: Pre-built binaries for each platform.
 - HIGH: Input via command-line parameters.
 - MEDIUM: Implement basic writing from XMP files to both `.glTF` and `.glb` files.
 - MEDIUM: JSON/XMP input via Pipe.
 - MEDIUM: Support for multiple packets.
 - LOW: Extraction of *KHR_xmp* metadata into a JSON file.

# Known issues

 - `xmp:MetadataDate` is not being updated or written.

# Version Notes

## 1.0.0-alpha01

 - Initial Release.
 - Support for `.glTF` 2.0 spec files.
 - Implements `--list` functionality, to list out *KHR_xmp* metadata contained in `.glTF` files.
 - Implements basic `--json` writing of *KHR_xmp* metadata.

## 1.0.0-alpha02

 - Full support for `.glb` files. Added example.

## 1.0.0-alpha03

 - Implemented support for the new *KHR_xmp_json_ld* extension replacing *KHR_xmp*.
 - The `--legacy` switch is required for all operations using *KHR_xmp*. This includes listing existing *KHR_xmp* data.
 - The `--migrate` switch has been added but is not usable yet. Will be released soon in the next release.
