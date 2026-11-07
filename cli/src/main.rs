use anyhow::Result;
use build_rs_libnix::process_buildrs_output;
use clap::Parser;
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[clap(
    name = "build-rs-libnix",
    about = "Parse the output of a build.rs script for 'nix build'"
)]

pub struct BuildRsNixArgs {
    /// Absolute path to the /nix/store/...-build-script-build.out file to parse
    #[clap(long = "script-output", value_name = "PATH")]
    pub script_output: PathBuf,

    /// A directory where the nix/* files are generated to
    #[clap(long = "out-dir", value_name = "PATH")]
    pub out_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = BuildRsNixArgs::parse();
    process_buildrs_output(&args.script_output, &args.out_dir)
}
