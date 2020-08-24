#![deny(warnings, missing_docs)]
//! This crate provides compile-time utilities for packaging 'cargo-web' build output
//! (rust compiled as wasm and associated html/css/etc files) inside native binaries
//! and is meant to be invoked from custom build.rs scripts
//!
//! Designed for use with the [`embed-wasm` crate](https://crates.io/crates/embed-wasm).
//! See [embed-wasm-example](https://github.com/inanna-malick/embed-wasm-example) for a full example.

use cargo_web::{CargoWebOpts, DeployOpts};
use ignore::Walk;
use std::env;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;


/// Compile the cargo executable in the 'wasm' subdir using cargo-web and
/// generate a static hashmap using 'phf' containing all static files
/// output by the cargo-web build process
pub fn compile_wasm<X: AsRef<Path>>(cargo_web_dir: X) {
    let profile = std::env::var("PROFILE").expect("expected env var PROFILE for build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("wasm_blobs_output_dir");
    // TODO: maybe not this? might wipe out resources, requiring extra recompile work?
    let _ = fs::remove_dir_all(&dest_path); // may already exist, nuke if that is the case
    fs::create_dir(&dest_path).unwrap();

    println!("dest path: {:?}", &dest_path);

    let current_dir = std::env::current_dir().unwrap();
    env::set_current_dir(current_dir.join(cargo_web_dir)).unwrap();

    //Build struct in DeployOpts is private so only way to create is this structopt method?
    let opts = if profile == "release" {
        DeployOpts::from_iter_safe(&[
            "--release",
            "--target=wasm32-unknown-unknown",
            "--output",
            dest_path.to_str().unwrap(),
        ])
    } else {
        DeployOpts::from_iter_safe(&[
            "--target=wasm32-unknown-unknown",
            "--output",
            dest_path.to_str().unwrap(),
        ])
    }
    .expect("expected hardcoded cargo-web args to be valid");

    cargo_web::run(CargoWebOpts::Deploy(opts)).unwrap();

    env::set_current_dir(current_dir).unwrap();

    let f_dest_path = Path::new(&out_dir).join("wasm_blobs.rs");
    let f = fs::File::create(&f_dest_path).unwrap();
    let mut file = BufWriter::new(f);

    let blobs: Vec<(String, PathBuf)> = (0..)
        .zip(Walk::new(dest_path.clone()))
        .filter_map(|(idx, result)| {
            // Each item yielded by the iterator is either a directory entry or an
            // error, so either print the path or the error.
            match result {
                Ok(entry) => {
                    if entry.metadata().unwrap().is_file() {
                        Some((format!("ENTRY_{}", idx), entry.into_path()))
                    } else {
                        None
                    }
                }
                Err(err) => {
                    eprintln!("error traversing wasm directory: {}", err);
                    None
                }
            }
        })
        .collect();

    for (identifier, path) in &blobs {
        writeln!(
            &mut file,
            "static {}: &'static [u8] = include_bytes!(\"{}\");",
            identifier,
            path.to_str().unwrap()
        )
        .unwrap();
    }


    let mut codegen = phf_codegen::Map::new();

    let dest_path = dest_path.to_str().unwrap();
    for (identifier, path) in &blobs {
        let key = &path.to_str().unwrap()[dest_path.len() + 1..];
        codegen.entry(key, identifier);
    }

    writeln!(
        &mut file,
        "static WASM: ::phf::Map<&'static str, &'static [u8]> =\n{};\n",
        codegen.build()
    ).unwrap();

    // register rerun-if-changed hooks for all wasm directory entries not in gitignore
    for result in Walk::new("wasm") {
        // Each item yielded by the iterator is either a directory entry or an
        // error, so either print the path or the error.
        match result {
            Ok(entry) => {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            }
            Err(err) => panic!("error traversing wasm directory: {}", err),
        }
    }

    // panic!("afaik only way to get println output from build.rs is to fail here");
}
