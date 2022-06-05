use cargo_metadata::{CargoOpt, MetadataCommand};
use clap::{AppSettings, Args, Parser, Subcommand};
use serde::Deserialize;

use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::str::FromStr;

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

    #[clap(long)]
    frozen: bool,

    #[clap(long)]
    locked: bool,

    #[clap(long)]
    offline: bool,
}

#[derive(Deserialize)]
struct PackageMetadata {
    coupler: Option<CouplerMetadata>,
}

#[derive(Deserialize)]
struct CouplerMetadata {
    #[serde(default)]
    formats: Vec<String>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Format {
    Vst3,
}

impl FromStr for Format {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vst3" => Ok(Format::Vst3),
            _ => Err(()),
        }
    }
}

fn main() {
    let Cargo::Coupler(cmd) = Cargo::parse();

    match cmd {
        Coupler::Bundle(cmd) => {
            let mut command = MetadataCommand::new();

            if let Some(manifest_path) = &cmd.manifest_path {
                command.manifest_path(manifest_path);
            }

            if cmd.no_default_features {
                command.features(CargoOpt::NoDefaultFeatures);
            }
            if cmd.all_features {
                command.features(CargoOpt::AllFeatures);
            }
            if !cmd.features.is_empty() {
                command.features(CargoOpt::SomeFeatures(cmd.features.clone()));
            }

            if cmd.frozen {
                command.other_options(vec!["--frozen".to_string()]);
            }
            if cmd.locked {
                command.other_options(vec!["--locked".to_string()]);
            }
            if cmd.offline {
                command.other_options(vec!["--offline".to_string()]);
            }

            let metadata = match command.exec() {
                Ok(metadata) => metadata,
                Err(error) => {
                    match error {
                        cargo_metadata::Error::CargoMetadata { stderr } => {
                            eprint!("{}", stderr);
                        }
                        _ => {
                            eprintln!("error: failed to invoke `cargo metadata`: {}", error);
                        }
                    }

                    process::exit(1);
                }
            };

            if !cmd.workspace && !cmd.exclude.is_empty() {
                eprintln!("error: --exclude can only be used together with --workspace");
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
                            "error: package `{}` not found in workspace `{}`",
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

            let mut formats = Vec::new();
            for format_str in &cmd.format {
                if let Ok(format) = Format::from_str(format_str) {
                    formats.push(format);
                } else {
                    eprintln!("error: invalid format `{}`", format_str);
                    process::exit(1);
                }
            }

            // Assemble a list of packages to build and bundles to create

            let mut packages_to_build = Vec::new();
            let mut packages_to_bundle = Vec::new();

            for &candidate in &candidates {
                let package = &metadata.packages[candidate];

                let has_cdylib =
                    package.targets.iter().any(|t| t.crate_types.iter().any(|c| c == "cdylib"));

                let package_metadata: Option<PackageMetadata> =
                    match serde_json::from_value(package.metadata.clone()) {
                        Ok(package_metadata) => package_metadata,
                        Err(err) => {
                            eprintln!(
                                "error: unable to parse [package.metadata.coupler] section: {}",
                                err
                            );
                            process::exit(1);
                        }
                    };

                if let Some(coupler_metadata) = package_metadata.and_then(|m| m.coupler) {
                    if !has_cdylib {
                        eprintln!("error: package `{}` has a [package.metadata.coupler] section but does not have a lib target of type cdylib", &package.name);
                        process::exit(1);
                    }

                    if coupler_metadata.formats.is_empty() {
                        eprintln!(
                            "warning: package `{}` does not specify any formats",
                            &package.name
                        );
                        continue;
                    }

                    let mut should_build = false;

                    for format_str in &coupler_metadata.formats {
                        let format = if let Ok(format) = Format::from_str(format_str) {
                            format
                        } else {
                            eprintln!(
                                "error: package `{}` specifies invalid format `{}`",
                                &package.name, format_str
                            );
                            process::exit(1);
                        };

                        if formats.is_empty() || formats.contains(&format) {
                            packages_to_bundle.push((candidate, format));
                            should_build = true;
                        }
                    }

                    if should_build {
                        packages_to_build.push(candidate);
                    }
                }
            }

            if packages_to_build.is_empty() || packages_to_bundle.is_empty() {
                eprintln!("error: no packages to bundle");
                process::exit(1);
            }

            // Invoke `cargo build`

            let cargo =
                env::var("CARGO").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("cargo"));

            let mut cargo_command = Command::new(cargo);
            cargo_command.arg("build");

            for &package in &packages_to_build {
                cargo_command.args(&["--package", &metadata.packages[package].name]);
            }

            if cmd.release {
                cargo_command.arg("--release");
            }
            if let Some(profile) = &cmd.profile {
                cargo_command.args(&["--profile", profile]);
            }

            if !cmd.features.is_empty() {
                cargo_command.arg("--features");
                for feature in cmd.features {
                    cargo_command.arg(feature);
                }
            }
            if cmd.all_features {
                cargo_command.arg("--all-features");
            }
            if cmd.no_default_features {
                cargo_command.arg("--no-default-features");
            }

            if let Some(target) = &cmd.target {
                cargo_command.args(&["--target", target]);
            }

            if let Some(target_dir) = &cmd.target_dir {
                cargo_command.arg("--target-dir");
                cargo_command.arg(target_dir);
            }

            if let Some(manifest_path) = &cmd.manifest_path {
                cargo_command.arg("--manifest-path");
                cargo_command.arg(manifest_path);
            }

            if cmd.frozen {
                cargo_command.arg("--frozen");
            }
            if cmd.locked {
                cargo_command.arg("--locked");
            }
            if cmd.offline {
                cargo_command.arg("--offline");
            }

            let result = cargo_command.spawn().and_then(|mut child| child.wait());
            if let Err(error) = result {
                eprintln!("error: failed to invoke `cargo build`: {}", error);
                process::exit(1);
            }

            if !result.unwrap().success() {
                process::exit(1);
            }
        }
    }
}
