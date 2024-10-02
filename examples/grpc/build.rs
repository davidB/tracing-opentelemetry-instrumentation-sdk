use std::path::PathBuf;

fn main() {
    // trigger rebuild if "proto" folder change
    print!("cargo:rerun-if-changed=./proto");

    //let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_dir = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("generated");
    std::fs::create_dir_all(&out_dir).unwrap();

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("helloworld_descriptor.bin"))
        .out_dir(out_dir)
        .compile_protos(&["helloworld.proto"], &["proto"])
        .unwrap();
}
