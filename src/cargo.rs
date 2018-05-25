use std::ffi;
use std::process;
use std::str;
use std::vec;

use failure;
use serde;
use serde_json;

const CURRENT_TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/current_target.txt"));

#[derive(Debug)]
pub struct Cargo {
    cmd: process::Command,
}

impl Cargo {
    pub fn new() -> Self {
        Self {
            cmd: process::Command::new("cargo"),
        }
    }

    pub fn arg<S: AsRef<ffi::OsStr>>(mut self, arg: S) -> Self {
        self.cmd.arg(arg);
        self
    }

    pub fn build(mut self) -> CargoBuild {
        self.cmd.arg("build").arg("--message-format=json");
        CargoBuild { cmd: self.cmd }
    }
}

pub struct CargoBuild {
    cmd: process::Command,
}

impl CargoBuild {
    pub fn new() -> Self {
        Cargo::new().build()
    }

    pub fn quiet(self) -> Self {
        self.arg("--quiet")
    }

    pub fn bin<S: AsRef<ffi::OsStr>>(self, name: S) -> Self {
        self.arg("--bin").arg(name)
    }

    pub fn example<S: AsRef<ffi::OsStr>>(self, name: S) -> Self {
        self.arg("--example").arg(name)
    }

    pub fn release(self) -> Self {
        self.arg("--release")
    }

    #[cfg(debug_assertions)]
    pub fn current_release(self) -> Self {
        self
    }

    #[cfg(not(debug_assertions))]
    pub fn current_release(self) -> Self {
        self.release()
    }

    pub fn target<S: AsRef<ffi::OsStr>>(self, triplet: S) -> Self {
        self.arg("--target").arg(triplet)
    }

    pub fn current_taget(self) -> Self {
        self.target(CURRENT_TARGET)
    }

    pub fn arg<S: AsRef<ffi::OsStr>>(mut self, arg: S) -> Self {
        self.cmd.arg(arg);
        self
    }

    pub fn exec(mut self) -> Result<MessageItr, failure::Error> {
        let output = self.cmd.output()?;
        if !output.status.success() {
            bail!("{}", String::from_utf8_lossy(&output.stderr));
        }

        let messages: Vec<Message> = str::from_utf8(&output.stdout)
            .expect("json to be UTF-8")
            .split('\n')
            .map(|s| Message {
                content: s.to_owned(),
            })
            .collect();

        Ok(messages.into_iter())
    }
}

pub type MessageItr = vec::IntoIter<Message>;

pub struct Message {
    content: String,
}

impl Message {
    pub fn convert<'a, T>(&'a self) -> Result<T, failure::Error>
    where
        T: serde::Deserialize<'a>,
    {
        let data = serde_json::from_str(self.content.as_str())?;
        Ok(data)
    }
}
