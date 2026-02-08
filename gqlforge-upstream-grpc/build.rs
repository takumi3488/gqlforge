use std::env::{set_var, var};
use std::path::{Path, PathBuf};

fn main() {
    let path = protoc_bin_vendored::protoc_bin_path().expect("Failed to find protoc binary");
    unsafe { set_var("PROTOC", format!("{}", path.display())) };

    let news = Path::new(gqlforge_fixtures::protobuf::NEWS);

    let out_dir = PathBuf::from(var("OUT_DIR").unwrap());

    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("news_descriptor.bin"))
        .compile_protos(&[&news], &[&news.parent().unwrap()])
        .unwrap();
}
