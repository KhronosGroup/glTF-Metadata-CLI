// SPDX-FileCopyrightText: 2020 2014-2020 The Khronos Group Inc.
//
// SPDX-License-Identifier: Apache-2.0

use crate::json_models::gltf::Gltf;
use crate::json_models::khr_xmp::KhrXmp;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

pub fn open_reader(path: &Path) -> Result<BufReader<File>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader)
}

pub fn read_json<T: Read>(reader: T) -> Result<KhrXmp, Box<dyn Error>> {
    let json = serde_json::from_reader(reader)?;
    Ok(json)
}

pub fn read_gltf<T: Read>(reader: T) -> Result<Gltf, Box<dyn Error>> {
    let gltf = serde_json::from_reader(reader)?;
    Ok(gltf)
}

pub fn open_writer(path: &Path) -> Result<BufWriter<File>, Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    Ok(writer)
}

pub fn write_gltf<T: Write>(writer: T, gltf: &Gltf) -> Result<(), Box<dyn Error>> {
    serde_json::to_writer_pretty(writer, gltf)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::read_to_string;

    const EXAMPLE_SIMPLE_JSON_PATH: &str = "./examples/sample.khr_xmp.json";
    const BOX_PATH: &str = "./examples/Box.gltf";
    const SCIFI_HELMET_PATH: &str = "./glTF-Sample-Models/2.0/SciFiHelmet/glTF/SciFiHelmet.gltf";

    #[test]
    fn can_open_and_read_json() {
        let path = Path::new(EXAMPLE_SIMPLE_JSON_PATH);

        let reader = open_reader(path);
        assert!(reader.is_ok());

        let json = read_json(reader.unwrap());
        assert!(json.is_ok());

        let expected = read_to_string(path).unwrap();

        // Actual needs an additional newline as sample.khr_xmp.json should have one.
        let actual = format!(
            "{}\n",
            serde_json::to_string_pretty(&json.unwrap()).unwrap()
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_open_and_read_gltf() {
        let path = Path::new(SCIFI_HELMET_PATH);
        let reader = open_reader(path);
        assert!(reader.is_ok());
        let gltf = read_gltf(reader.unwrap());
        assert!(gltf.is_ok());
    }
}
