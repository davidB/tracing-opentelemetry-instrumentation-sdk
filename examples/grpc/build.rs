fn main() {
    tonic_build::configure()
        .compile(&["example.proto"], &["."])
        .unwrap();
}
