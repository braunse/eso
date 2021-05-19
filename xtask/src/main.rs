use std::{env, ffi::OsString, path::PathBuf};

use argh::FromArgs;
use colored::Colorize;
use eyre::Result;
use lazy_static::lazy_static;
use tap::Tap;
use xshell::{cmd, mkdir_p, pushd, pushenv, rm_rf};

use crate::utils::amend_env_var;

mod utils;

const XTASK_DIR: &'static str = env!("CARGO_MANIFEST_DIR");

lazy_static! {
    static ref WORKSPACE_ROOT: PathBuf = {
        PathBuf::from(XTASK_DIR)
            .parent()
            .expect("Could not find parent of `xtask` directory")
            .to_path_buf()
    };
    static ref PROF_DIR: PathBuf = {
        let mut r = WORKSPACE_ROOT.clone();
        r.push("_prof");
        r
    };
}

#[derive(FromArgs, Debug)]
/// Build and maintenance task automation for the `eso` crate.
struct Args {
    #[argh(subcommand)]
    command: Command,
}

impl Args {
    fn run(&self) -> Result<()> {
        self.command.run(self)
    }
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Command {
    CoverTest(CoverTest),
}

impl Command {
    fn run(&self, args: &Args) -> Result<()> {
        match self {
            Command::CoverTest(ct) => ct.run(args),
        }
    }
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "cover-test")]
/// Run tests with coverage enable
struct CoverTest {
    #[argh(switch)]
    /// compile and run the tests in release mode
    release: bool,

    #[argh(positional)]
    /// arguments that will be forwarded to `cargo test`
    args: Vec<OsString>,

    #[argh(switch)]
    /// generate `lcov.info` file
    lcov: bool,

    #[argh(switch)]
    /// generate HTML report
    html: bool,
}

impl CoverTest {
    fn run(&self, _args: &Args) -> Result<()> {
        println!(
            "* {} {}\n\n",
            "Workspace root is at".green(),
            WORKSPACE_ROOT.to_string_lossy().blue()
        );
        let _ = pushd(&*WORKSPACE_ROOT)?;
        let _rustflags = amend_env_var("RUSTFLAGS", " ", "-Zinstrument-coverage")?;

        println!("\n\n* {}\n\n", "Building with coverage enabled".green());
        let release_switch = if self.release {
            Some("--release")
        } else {
            None
        };
        cmd!("cargo +nightly build {release_switch...}").run()?;
        println!("* {}\n\n", "Running with profile data collection".green());

        rm_rf(&*PROF_DIR)?;
        mkdir_p(&*PROF_DIR)?;
        let _profile_file = pushenv(
            "LLVM_PROFILE_FILE",
            PROF_DIR
                .to_path_buf()
                .tap_mut(|p| p.push("profile-%p-%m.profraw")),
        );
        let passthru = &self.args;
        cmd!("cargo +nightly test {release_switch...} {passthru...}").run()?;
        println!("* {}\n\n", "Running with profile data collection".green());

        if self.lcov {
            let profile = if self.release { "release" } else { "debug" };
            cmd!("grcov . -s . --binary-path ./target/{profile} -t lcov -o ./lcov.info").run()?;
        }

        if self.html {
            let profile = if self.release { "release" } else { "debug" };
            cmd!("grcov . -s . --binary-path ./target/{profile} -t html -o ./_cov").run()?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args: Args = argh::from_env();
    args.run()
}
