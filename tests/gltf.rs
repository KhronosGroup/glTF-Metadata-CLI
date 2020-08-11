#[test]
fn gltf_files_preserved() {
    // TODO: Can currently manually do this using jq, but would be good to have a unit
    //   test in order to quickly test this out. Issue is dealing with the BufWriter.

    // Was used early on for validation, figure it will be helpful with making this test.
    /*let input_path = Path::new("./glTF-Sample-Models/2.0/Box/glTF/Box.gltf");
    let input_reader = open_reader(input_path).unwrap();
    let gltf = read_gltf(input_reader).unwrap();

    let output_path = Path::new("~/output.gltf");
    let output_writer = open_writer(output_path).unwrap();
    write_gltf(output_writer, &gltf).unwrap()*/
}
