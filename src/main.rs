use std::fs::{self, create_dir_all};
use std::process;

use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
enum Command {
    Roogle(Opt),
}

#[derive(Debug, StructOpt)]
struct Opt {}

fn build(_: &Opt) -> Result<()> {
    process::Command::new("cargo")
        .args(&[
            "+nightly",
            "rustdoc",
            "--",
            "--output-format=json",
            "-Z",
            "unstable-options",
        ])
        .spawn()
        .context("failed to build search index using `rustdoc`")?
        .wait()?;

    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("failed to get crate's metadata")?;
    let members = metadata
        .packages
        .iter()
        .filter(|p| metadata.workspace_members.contains(&p.id))
        .map(|p| &p.name);

    create_dir_all("target/roogle/crate").context("failed create `crate` directory")?;
    for member in members {
        fs::rename(
            format!("target/doc/{}.json", member),
            format!("target/roogle/crate/{}.json", member),
        )
        .context("failed to store crate's index to index folder")?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let Command::Roogle(opt) = Command::from_args();
    build(&opt)?;

    process::Command::new("roogle")
        .args(&["--index", "target/roogle"])
        .spawn()
        .context("failed to start server")?
        .wait()?;

    Ok(())
}
