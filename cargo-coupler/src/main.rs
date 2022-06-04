use cargo_metadata::{MetadataCommand};
use clap::{AppSettings, Args, Parser, Subcommand};

use std::collections::{HashMap, HashSet};
use std::process;

#[derive(Parser)]
#[clap(bin_name = "cargo")]
enum Cargo {
    #[clap(subcommand)]
    Coupler(Coupler),
}

#[derive(Subcommand)]
#[clap(version, about, long_about = None)]
enum Coupler {
    Bundle(Bundle),
}

#[derive(Args, Debug)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
struct Bundle {
    #[clap(long, short, value_name = "PACKAGE")]
    package: Vec<String>,

    #[clap(long)]
    workspace: bool,

    #[clap(long, value_name = "PACKAGE")]
    exclude: Vec<String>,

    #[clap(long, short, value_name = "FORMAT")]
    format: Vec<String>,

    #[clap(long, short)]
    release: bool,

    #[clap(long, value_name = "PROFILE-NAME")]
    profile: Option<String>,

    #[clap(long, value_name = "FEATURES", multiple_values = true)]
    features: Vec<String>,

    #[clap(long)]
    all_features: bool,

    #[clap(long)]
    no_default_features: bool,

    #[clap(long, value_name = "TRIPLE")]
    target: Option<String>,

    #[clap(long, value_name = "DIRECTORY")]
    target_dir: Option<std::path::PathBuf>,

    #[clap(long, parse(from_os_str))]
    manifest_path: Option<std::path::PathBuf>,
}

fn main() {
    let Cargo::Coupler(cmd) = Cargo::parse();

    match cmd {
        Coupler::Bundle(cmd) => {
            let mut command = MetadataCommand::new();
            if let Some(manifest_path) = &cmd.manifest_path {
                command.manifest_path(manifest_path);
            }
            let metadata = command.exec().unwrap();

            if !cmd.workspace && !cmd.exclude.is_empty() {
                eprintln!("--exclude can only be used together with --workspace");
                process::exit(1);
            }

            let mut packages_by_id = HashMap::new();
            for (index, package) in metadata.packages.iter().enumerate() {
                packages_by_id.insert(package.id.clone(), index);
            }

            // Build a list of candidate packages for bundling
            let mut candidates = Vec::new();
            if cmd.workspace {
                let mut exclude = HashSet::new();
                for package_name in &cmd.exclude {
                    exclude.insert(package_name);
                }

                for package_id in &metadata.workspace_members {
                    let package_index = packages_by_id[package_id];
                    if !exclude.contains(&metadata.packages[package_index].name) {
                        candidates.push(package_index);
                    }
                }
            } else if !cmd.package.is_empty() {
                // Build an index of packages in the current workspace by name
                let mut packages_by_name = HashMap::new();
                for package_id in &metadata.workspace_members {
                    let package_index = packages_by_id[package_id];
                    packages_by_name
                        .insert(metadata.packages[package_index].name.clone(), package_index);
                }

                for package_name in &cmd.package {
                    if let Some(&package_index) = packages_by_name.get(package_name) {
                        candidates.push(package_index);
                    } else {
                        eprintln!(
                            "package `{}` not found in workspace `{}`",
                            package_name, &metadata.workspace_root
                        );
                        process::exit(1);
                    }
                }
            } else if let Some(root) = &metadata.resolve.as_ref().unwrap().root {
                // If neither --workspace nor --package is specified and there is a
                // root package, just try to build the root
                candidates.push(packages_by_id[root]);
            } else {
                // If there is no root package, search the entire workspace
                for package_id in &metadata.workspace_members {
                    candidates.push(packages_by_id[package_id]);
                }
            }

            for &candidate in &candidates {
                dbg!(&metadata.packages[candidate].id);
            }
        }
    }
}
