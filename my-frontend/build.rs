use cargo_web::{CargoWebOpts, DeployOpts};
use ignore::Walk;
use std::env;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

// NOTE: currently only tested with flat deploy dir
fn main() {
    let profile = std::env::var("PROFILE").expect("expected env var PROFILE for build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("wasm_blobs_output_dir");
    // TODO: maybe not this? might wipe out resources, requiring extra recompile work?
    let _ = fs::remove_dir_all(&dest_path); // may already exist, nuke if that is the case
    fs::create_dir(&dest_path).unwrap();

    println!("dest path: {:?}", &dest_path);

    let current_dir = std::env::current_dir().unwrap();
    env::set_current_dir(current_dir.join("wasm")).unwrap();

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

    write!(&mut file, "use phf::phf_map;\n").unwrap();

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
        write!(
            &mut file,
            "static {}: &'static [u8] = include_bytes!(\"{}\");\n",
            identifier,
            path.to_str().unwrap()
        ).unwrap();
    }

    write!(
        &mut file,
        "static WASM: phf::Map<&'static str, &'static [u8]> =\n"
    ).unwrap();

    write!(&mut file, "phf_map! {{\n").unwrap();

    let dest_path = dest_path.to_str().unwrap();
    for (identifier, path) in &blobs {
        let key = &path.to_str().unwrap()[dest_path.len() + 1..];
        write!(
            &mut file,
            "  \"{}\" => {},\n",
            key,
            identifier
        ).unwrap();
    }

    write!(&mut file, "}};\n").unwrap();

    // register rerun-if-changed hooks for all wasm directory entries not in gitignore
    for result in Walk::new("wasm") {
        // Each item yielded by the iterator is either a directory entry or an
        // error, so either print the path or the error.
        match result {
            Ok(entry) => {
                if entry.metadata().unwrap().is_file() {
                    println!("cargo:rerun-if-changed={}", entry.path().display());
                }
            }
            Err(err) => panic!("error traversing wasm directory: {}", err),
        }
    }

    // panic!("afaik only way to get println output from build.rs is to fail here");
}
