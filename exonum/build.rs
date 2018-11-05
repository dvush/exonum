// spell-checker:ignore rustc

extern crate protoc_rust;

use std::{env, fs::File, io::Write, path::Path, process::Command};

use protoc_rust::Customize;

static USER_AGENT_FILE_NAME: &str = "user_agent";

fn main() {
    let package_name = option_env!("CARGO_PKG_NAME").unwrap_or("exonum");
    let package_version = option_env!("CARGO_PKG_VERSION").unwrap_or("?");
    let rust_version = rust_version().unwrap_or("rust ?".to_string());
    let user_agent = format!("{} {}/{}\n", package_name, package_version, rust_version);

    let out_dir = env::var("OUT_DIR").expect("Unable to get OUT_DIR");
    let dest_path = Path::new(&out_dir).join(USER_AGENT_FILE_NAME);
    let mut file = File::create(dest_path).expect("Unable to create output file");
    file.write_all(user_agent.as_bytes())
        .expect("Unable to data to file");

    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/encoding/protobuf",
        input: &[
            "src/encoding/protobuf/proto/helpers.proto",
            "src/encoding/protobuf/proto/blockchain.proto",
            "src/encoding/protobuf/proto/protocol.proto",
        ],
        includes: &["src/encoding/protobuf/proto"],
        customize: Customize {
            ..Default::default()
        },
    }).expect("protoc");
}

fn rust_version() -> Option<String> {
    let rustc = option_env!("RUSTC").unwrap_or("rustc");

    let output = Command::new(rustc).arg("-V").output().ok()?.stdout;
    String::from_utf8(output).ok()
}
