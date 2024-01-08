use cargo_metadata::{CargoOpt, MetadataCommand};
use clap::{AppSettings, Args, Parser, Subcommand};
use serde::Deserialize;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{self, Command};
use std::str::{self, FromStr};

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
    name: Option<String>,

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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Arch {
    Aarch64,
    I686,
    X86_64,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Os {
    Linux,
    MacOs,
    Windows,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Target {
    arch: Arch,
    os: Os,
}

impl FromStr for Target {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "aarch64-apple-darwin" => Ok(Target {
                arch: Arch::Aarch64,
                os: Os::MacOs,
            }),
            "i686-pc-windows-gnu" => Ok(Target {
                arch: Arch::I686,
                os: Os::Windows,
            }),
            "i686-pc-windows-msvc" => Ok(Target {
                arch: Arch::I686,
                os: Os::Windows,
            }),
            "i686-unknown-linux-gnu" => Ok(Target {
                arch: Arch::I686,
                os: Os::Linux,
            }),
            "x86_64-apple-darwin" => Ok(Target {
                arch: Arch::X86_64,
                os: Os::MacOs,
            }),
            "x86_64-pc-windows-gnu" => Ok(Target {
                arch: Arch::X86_64,
                os: Os::MacOs,
            }),
            "x86_64-pc-windows-msvc" => Ok(Target {
                arch: Arch::X86_64,
                os: Os::MacOs,
            }),
            "x86_64-unknown-linux-gnu" => Ok(Target {
                arch: Arch::X86_64,
                os: Os::Linux,
            }),
            _ => Err(()),
        }
    }
}

struct PackageInfo {
    index: usize,
    name: String,
    formats: Vec<Format>,
}

fn main() {
    let Cargo::Coupler(cmd) = Cargo::parse();

    match cmd {
        Coupler::Bundle(cmd) => {
            // Query `rustc` for host target if no --target argument was given

            let target_str = if let Some(target) = &cmd.target {
                target.clone()
            } else {
                let rustc_path =
                    env::var("RUSTC").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("rustc"));
                let mut rustc = Command::new(rustc_path);
                rustc.args(&["--version", "--verbose"]);

                let result = rustc.output();
                if let Err(error) = result {
                    eprintln!(
                        "error: failed to invoke `rustc` to query host information: {}",
                        error
                    );
                    process::exit(1);
                }

                let output = result.unwrap();
                if !output.status.success() {
                    eprintln!("error: failed to invoke `rustc` to query host information");
                    eprintln!();
                    io::stderr().write_all(&output.stderr).unwrap();
                    process::exit(1);
                }

                const HOST_FIELD: &str = "host: ";
                let output_str = str::from_utf8(&output.stdout).unwrap();
                let host = output_str
                    .lines()
                    .find(|l| l.starts_with(HOST_FIELD))
                    .map(|l| &l[HOST_FIELD.len()..]);
                if host.is_none() {
                    eprintln!("error: failed to invoke `rustc` to query host information");
                    process::exit(1);
                }
                host.unwrap().to_string()
            };

            // Extract arch and OS from target triple

            let target = if let Ok(target) = Target::from_str(&target_str) {
                target
            } else {
                eprintln!("error: unsupported target `{}`", &target_str);
                process::exit(1);
            };

            // Invoke `cargo metadata`

            let mut command = MetadataCommand::new();

            command.other_options(vec!["--filter-platform".to_string(), target_str.clone()]);

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

            // Build the actual list of packages to bundle

            let mut packages_to_build = Vec::new();
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

                    let mut package_formats = Vec::new();
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
                            package_formats.push(format);
                        }
                    }

                    if !package_formats.is_empty() {
                        packages_to_build.push(PackageInfo {
                            index: candidate,
                            name: coupler_metadata.name.as_ref().unwrap_or(&package.name).clone(),
                            formats: package_formats,
                        });
                    }
                }
            }

            if packages_to_build.is_empty() {
                eprintln!("error: no packages to bundle");
                process::exit(1);
            }

            // Invoke `cargo build`

            let cargo_path =
                env::var("CARGO").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("cargo"));
            let mut cargo = Command::new(cargo_path);
            cargo.arg("build");

            for package_info in &packages_to_build {
                cargo.args(&["--package", &metadata.packages[package_info.index].name]);
            }

            cargo.arg("--lib");

            if cmd.release {
                cargo.arg("--release");
            }
            if let Some(profile) = &cmd.profile {
                cargo.args(&["--profile", profile]);
            }

            if !cmd.features.is_empty() {
                cargo.arg("--features");
                for feature in cmd.features {
                    cargo.arg(feature);
                }
            }
            if cmd.all_features {
                cargo.arg("--all-features");
            }
            if cmd.no_default_features {
                cargo.arg("--no-default-features");
            }

            if let Some(target) = &cmd.target {
                cargo.args(&["--target", target]);
            }

            if let Some(target_dir) = &cmd.target_dir {
                cargo.arg("--target-dir");
                cargo.arg(target_dir);
            }

            if let Some(manifest_path) = &cmd.manifest_path {
                cargo.arg("--manifest-path");
                cargo.arg(manifest_path);
            }

            if cmd.frozen {
                cargo.arg("--frozen");
            }
            if cmd.locked {
                cargo.arg("--locked");
            }
            if cmd.offline {
                cargo.arg("--offline");
            }

            let result = cargo.spawn().and_then(|mut child| child.wait());
            if let Err(error) = result {
                eprintln!("error: failed to invoke `cargo build`: {}", error);
                process::exit(1);
            }

            if !result.unwrap().success() {
                process::exit(1);
            }

            // Create bundles

            let target_dir = if let Some(target_dir) = &cmd.target_dir {
                target_dir
            } else {
                metadata.target_directory.as_std_path()
            };

            let profile = if let Some(profile) = &cmd.profile {
                profile
            } else if cmd.release {
                "release"
            } else {
                "dev"
            };

            let mut binary_dir = PathBuf::from(target_dir);
            if let Some(target) = &cmd.target {
                binary_dir.push(target);
            }
            binary_dir.push(if profile == "dev" { "debug" } else { profile });

            for package_info in &packages_to_build {
                for format in &package_info.formats {
                    match format {
                        Format::Vst3 => {
                            let mut bundle_path = binary_dir.join("bundle");
                            bundle_path.push(format!("{}.vst3", &package_info.name));

                            if bundle_path.exists() {
                                fs::remove_dir_all(&bundle_path).unwrap();
                            }

                            let mut dst_dir = bundle_path.clone();
                            dst_dir.push("Contents");

                            let arch_str = match target.arch {
                                Arch::Aarch64 => unimplemented!(),
                                Arch::I686 => "x86",
                                Arch::X86_64 => "x86_64",
                            };
                            let os_str = match target.os {
                                Os::Linux => unimplemented!(),
                                Os::MacOs => unimplemented!(),
                                Os::Windows => "win",
                            };
                            dst_dir.push(format!("{}-{}", arch_str, os_str));

                            fs::create_dir_all(&dst_dir).unwrap();

                            let src_filename = match target.os {
                                Os::Linux => unimplemented!(),
                                Os::MacOs => unimplemented!(),
                                Os::Windows => {
                                    format!("{}.dll", &metadata.packages[package_info.index].name)
                                }
                            };
                            let src = binary_dir.join(&src_filename);

                            let dst_filename = match target.os {
                                Os::Linux => unimplemented!(),
                                Os::MacOs => unimplemented!(),
                                Os::Windows => format!("{}.vst3", &package_info.name),
                            };
                            let dst = dst_dir.join(&dst_filename);

                            fs::copy(&src, &dst).unwrap();
                        }
                    }
                }
            }
        }
    }
}
