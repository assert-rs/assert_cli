use std::ffi;
use std::fs;
use std::io;
use std::io::Write;
use std::path;
use std::process;

use globwalk;
use tempfile;
use failure;

// Quick and dirty for doc tests; not meant for long term use.
pub use tempfile::TempDir;

/// Extend `TempDir` to perform operations on relative paths within the temp directory via
/// `ChildPath`.
pub trait TempDirChildExt {
    /// Create a path within the temp directory.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use assert_cli::temp::*;
    ///
    /// let temp = TempDir::new("TempDirChildExt_demo").unwrap();
    /// println!("{:?}", temp.path());
    /// println!("{:?}", temp.child("foo/bar.txt").path());
    /// temp.close().unwrap();
    /// ```
    fn child<P>(&self, path: P) -> ChildPath
    where
        P: AsRef<path::Path>;
}

impl TempDirChildExt for tempfile::TempDir {
    fn child<P>(&self, path: P) -> ChildPath
    where
        P: AsRef<path::Path>,
    {
        ChildPath::new(self.path().join(path.as_ref()))
    }
}

/// A path within a TempDir
pub struct ChildPath {
    path: path::PathBuf,
}

impl ChildPath {
    /// Wrap a path for use with special built extension traits.
    ///
    /// See trait implementations or `TempDirChildExt` for more details.
    pub fn new<P>(path: P) -> Self
    where
        P: Into<path::PathBuf>,
    {
        Self { path: path.into() }
    }

    /// Access the path.
    pub fn path(&self) -> &path::Path {
        &self.path
    }
}

/// Extend `TempDir` to run commands in it.
pub trait TempDirCommandExt {
    /// Constructs a new Command for launching the program at path program, with the following
    /// default configuration:
    ///
    /// - The current working directory is the temp dir
    /// - No arguments to the program
    /// - Inherit the current process's environment
    /// - Inherit the current process's working directory
    /// - Inherit stdin/stdout/stderr for spawn or status, but create pipes for output
    /// - Builder methods are provided to change these defaults and otherwise configure the process.
    ///
    /// If program is not an absolute path, the PATH will be searched in an OS-defined way.
    ///
    /// The search path to be used may be controlled by setting the PATH environment variable on
    /// the Command, but this has some implementation limitations on Windows (see
    /// https://github.com/rust-lang/rust/issues/37519).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use assert_cli::temp::*;
    ///
    /// let temp = TempDir::new("TempDirChildExt_demo").unwrap();
    /// temp.command("pwd").output().unwrap();
    /// temp.close().unwrap();
    /// ```
    fn command<S>(&self, program: S) -> process::Command
    where
        S: AsRef<ffi::OsStr>;
}

impl TempDirCommandExt for tempfile::TempDir {
    fn command<S>(&self, program: S) -> process::Command
    where
        S: AsRef<ffi::OsStr>,
    {
        let mut cmd = process::Command::new(program);
        cmd.current_dir(self.path());
        cmd
    }
}

impl TempDirCommandExt for ChildPath {
    fn command<S>(&self, program: S) -> process::Command
    where
        S: AsRef<ffi::OsStr>,
    {
        let mut cmd = process::Command::new(program);
        cmd.current_dir(self.path());
        cmd
    }
}

/// Extend `ChildPath` to create empty files.
pub trait ChildPathTouchExt {
    /// Create an empty file at `ChildPath`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use assert_cli::temp::*;
    ///
    /// let temp = TempDir::new("TempDirChildExt_demo").unwrap();
    /// temp.child("foo.txt").touch().unwrap();
    /// temp.close().unwrap();
    /// ```
    fn touch(&self) -> io::Result<()>;
}

impl ChildPathTouchExt for ChildPath {
    fn touch(&self) -> io::Result<()> {
        touch(self.path())
    }
}

/// Extend `ChildPath` to write binary files.
pub trait ChildPathWriteBinExt {
    /// Write a binary file at `ChildPath`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use assert_cli::temp::*;
    ///
    /// let temp = TempDir::new("TempDirChildExt_demo").unwrap();
    /// temp.child("foo.txt").write_binary(b"To be or not to be...").unwrap();
    /// temp.close().unwrap();
    /// ```
    fn write_binary(&self, data: &[u8]) -> io::Result<()>;
}

impl ChildPathWriteBinExt for ChildPath {
    fn write_binary(&self, data: &[u8]) -> io::Result<()> {
        write_binary(self.path(), data)
    }
}

/// Extend `ChildPath` to write text files.
pub trait ChildPathWriteStrExt {
    /// Write a text file at `ChildPath`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use assert_cli::temp::*;
    ///
    /// let temp = TempDir::new("TempDirChildExt_demo").unwrap();
    /// temp.child("foo.txt").write_str("To be or not to be...").unwrap();
    /// temp.close().unwrap();
    /// ```
    fn write_str(&self, data: &str) -> io::Result<()>;
}

impl ChildPathWriteStrExt for ChildPath {
    fn write_str(&self, data: &str) -> io::Result<()> {
        write_str(self.path(), data)
    }
}

/// Extend `TempDir` to copy files into it.
pub trait TempDirCopyExt {
    /// Copy files and directories into the current path from the `source` according to the glob
    /// `patterns`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use assert_cli::temp::*;
    ///
    /// let temp = TempDir::new("TempDirChildExt_demo").unwrap();
    /// temp.copy_from(".", &["*.rs"]).unwrap();
    /// temp.close().unwrap();
    /// ```
    fn copy_from<P, S>(&self, source: P, patterns: &[S]) -> Result<(), failure::Error>
    where
        P: AsRef<path::Path>,
        S: AsRef<str>;
}

impl TempDirCopyExt for tempfile::TempDir {
    fn copy_from<P, S>(&self, source: P, patterns: &[S]) -> Result<(), failure::Error>
    where
        P: AsRef<path::Path>,
        S: AsRef<str>,
    {
        copy_from(self.path(), source.as_ref(), patterns)
    }
}

impl TempDirCopyExt for ChildPath {
    fn copy_from<P, S>(&self, source: P, patterns: &[S]) -> Result<(), failure::Error>
    where
        P: AsRef<path::Path>,
        S: AsRef<str>,
    {
        copy_from(self.path(), source.as_ref(), patterns)
    }
}

fn touch(path: &path::Path) -> io::Result<()> {
    fs::File::create(path)?;
    Ok(())
}

fn write_binary(path: &path::Path, data: &[u8]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(data)?;
    Ok(())
}

fn write_str(path: &path::Path, data: &str) -> io::Result<()> {
    write_binary(path, data.as_bytes())
}

fn copy_from<S>(
    target: &path::Path,
    source: &path::Path,
    patterns: &[S],
) -> Result<(), failure::Error>
where
    S: AsRef<str>,
{
    for entry in globwalk::GlobWalker::from_patterns(patterns, source)?.follow_links(true) {
        let entry = entry?;
        let rel = entry
            .path()
            .strip_prefix(source)
            .expect("entries to be under `source`");
        let target_path = target.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(target_path)?;
        } else if entry.file_type().is_file() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
