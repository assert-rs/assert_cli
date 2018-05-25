use std::env;
use std::fs;
use std::io::Write;
use std::path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // env::ARCH doesn't include full triple, and AFAIK there isn't a nicer way of getting the full triple
    // (see lib.rs for the rest of this hack)
    let out = path::PathBuf::from(env::var_os("OUT_DIR").expect("run within cargo"))
        .join("current_target.txt");
    let default_target = env::var("TARGET").expect("run as cargo build script");
    let mut file = fs::File::create(out).unwrap();
    file.write_all(default_target.as_bytes()).unwrap();
}
