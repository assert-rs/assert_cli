#[macro_use]
extern crate error_chain;

use std::env;
use std::process;

error_chain! {
    foreign_links {
        Env(env::VarError);
        ParseInt(std::num::ParseIntError);
    }
}

fn run() -> Result<()> {
    if let Ok(text) = env::var("stdout") {
        println!("{}", text);
    }
    if let Ok(text) = env::var("stderr") {
        eprintln!("{}", text);
    }

    let code = env::var("exit")
        .ok()
        .map(|v| v.parse::<i32>())
        .map_or(Ok(None), |r| r.map(Some))
        .chain_err(|| "Invalid exit code")?
        .unwrap_or(0);
    process::exit(code);
}

quick_main!(run);
