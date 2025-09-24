use std::error::Error;
use std::process::{Command, Stdio};

// This build script uses Buf to download protobuf dependencies and generate
// server and client code using the Tonic build crate. Unfortunately, the
// generated code does not use the protobuf hierarchy as modules in the code.
// It would also be very nice if this could be encapsulated in the Tonic crates.
fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=proto");
    println!("cargo::rerun-if-changed=buf.lock");
    // Export the entire protobuf dependency tree into exploded directories.
    let path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let _ = Command::new("buf")
        .args(["export", "--output", path.to_str().unwrap()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .spawn()?
        .wait()
        .expect("failed to buf export");

    // Get the list of protobufs filenames in the exploded directories.
    let mut command = Command::new("buf");
    let output = command
        .args(["ls-files", path.to_str().unwrap()])
        .stdout(Stdio::piped())
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .spawn()
        .expect("failed to buf ls-files");

    // Split the command output into a vector of filename strings.
    let output = output.wait_with_output().unwrap();
    let files = String::from_utf8(output.stdout).unwrap();
    let files = files
        .split('\n')
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<&str>>();

    // Compile protobufs and generate Rust server and client code (gRPC).
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .build_transport(true)
        .compile_protos(&files, &[path.to_str().unwrap()])?;

    Ok(())
}
