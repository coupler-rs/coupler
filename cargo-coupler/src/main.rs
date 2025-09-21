use cargo_metadata::{CargoOpt, Metadata, MetadataCommand};
use clap::{AppSettings, Args, Parser, Subcommand};
use serde::Deserialize;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::{self, Display};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
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
    Clap,
    Vst3,
}

impl Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match *self {
            Format::Clap => "clap",
            Format::Vst3 => "vst3",
        };

        f.write_str(name)
    }
}

impl FromStr for Format {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "clap" => Ok(Format::Clap),
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
    Aarch64X86_64, // Universal binary containing both aarch64 and x86_64
}

#[allow(clippy::enum_variant_names)]
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
            "aarch64-x86_64-apple-darwin" => Ok(Target {
                arch: Arch::Aarch64X86_64,
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
                os: Os::Windows,
            }),
            "x86_64-pc-windows-msvc" => Ok(Target {
                arch: Arch::X86_64,
                os: Os::Windows,
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
    package_name: String,
    lib_name: String,
    name: String,
    formats: Vec<Format>,
}

fn main() {
    let Cargo::Coupler(cmd) = Cargo::parse();

    match cmd {
        Coupler::Bundle(cmd) => {
            bundle(&cmd);
        }
    }
}

fn out_dir_for_target(cmd: &Bundle, metadata: &Metadata, target: Option<&str>) -> PathBuf {
    let target_dir = if let Some(target_dir) = &cmd.target_dir {
        target_dir
    } else {
        metadata.target_directory.as_std_path()
    };
    let mut out_dir = PathBuf::from(target_dir);

    if let Some(target) = target {
        out_dir.push(target);
    }

    let profile = if let Some(profile) = &cmd.profile {
        profile
    } else if cmd.release {
        "release"
    } else {
        "debug"
    };
    out_dir.push(profile);

    out_dir
}

fn bundle(cmd: &Bundle) {
    // Query `rustc` for host target if no --target argument was given

    let target_str = if let Some(target) = &cmd.target {
        target.clone()
    } else {
        let rustc_path =
            env::var("RUSTC").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("rustc"));
        let mut rustc = Command::new(rustc_path);
        rustc.args(["--version", "--verbose"]);

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

    if target.arch == Arch::Aarch64X86_64 {
        command.other_options(vec![
            "--filter-platform".to_string(),
            "aarch64-apple-darwin".to_string(),
            "--filter-platform".to_string(),
            "x86_64-apple-darwin".to_string(),
        ]);
    } else {
        command.other_options(vec!["--filter-platform".to_string(), target_str.clone()]);
    }

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

    let mut packages_by_name = HashMap::new();
    for package_id in &metadata.workspace_members {
        let package_index = packages_by_id[package_id];
        packages_by_name.insert(metadata.packages[package_index].name.clone(), package_index);
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

    let mut formats_to_build = Vec::new();
    for format_str in &cmd.format {
        if let Ok(format) = Format::from_str(format_str) {
            formats_to_build.push(format);
        } else {
            eprintln!("error: invalid format `{}`", format_str);
            process::exit(1);
        }
    }

    // Build the actual list of packages to bundle

    let mut packages_to_build = Vec::new();
    for &candidate in &candidates {
        let package = &metadata.packages[candidate];

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
            if coupler_metadata.formats.is_empty() {
                eprintln!(
                    "warning: package `{}` does not specify any formats",
                    &package.name
                );
                continue;
            }

            let mut formats = Vec::new();
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

                if formats_to_build.is_empty() || formats_to_build.contains(&format) {
                    formats.push(format);
                }
            }

            let has_cdylib =
                package.targets.iter().any(|t| t.crate_types.iter().any(|c| c == "cdylib"));
            if !has_cdylib {
                eprintln!(
                    "error: package `{}` does not have a lib target of type cdylib",
                    package.name
                );
                process::exit(1);
            }

            let crate_name = package.name.replace('-', "_");
            let lib_name = match target.os {
                Os::Linux => format!("lib{crate_name}.so"),
                Os::MacOs => format!("lib{crate_name}.dylib"),
                Os::Windows => format!("{crate_name}.dll"),
            };

            packages_to_build.push(PackageInfo {
                package_name: package.name.to_owned(),
                lib_name,
                name: coupler_metadata.name.as_ref().unwrap_or(&package.name).clone(),
                formats,
            });
        }
    }

    if packages_to_build.is_empty() {
        eprintln!("error: no packages to bundle");
        process::exit(1);
    }

    let out_dir = out_dir_for_target(cmd, &metadata, cmd.target.as_deref());

    // Invoke `cargo build`

    if target.arch == Arch::Aarch64X86_64 {
        build_universal(
            cmd,
            &metadata,
            &["aarch64-apple-darwin", "x86_64-apple-darwin"],
            &packages_to_build,
            &out_dir,
        );
    } else {
        build(cmd, cmd.target.as_deref(), &packages_to_build);
    }

    // Create bundles

    for package_info in &packages_to_build {
        for format in &package_info.formats {
            match format {
                Format::Clap => {
                    bundle_clap(package_info, &out_dir, &target);
                }
                Format::Vst3 => {
                    bundle_vst3(package_info, &out_dir, &target);
                }
            }
        }
    }
}

fn build_universal(
    cmd: &Bundle,
    metadata: &Metadata,
    targets: &[&str],
    packages: &[PackageInfo],
    out_dir: &Path,
) {
    for target in targets {
        build(cmd, Some(target), packages);
    }

    fs::create_dir_all(out_dir).unwrap();

    for package_info in packages {
        let out_lib = out_dir.join(&package_info.lib_name);

        let mut lipo = Command::new("lipo");
        lipo.arg("-create").arg("-output").arg(out_lib);

        for target in targets {
            let lib_name = &package_info.lib_name;
            let input_lib = out_dir_for_target(cmd, metadata, Some(target)).join(lib_name);
            lipo.arg(input_lib);
        }

        let result = lipo.spawn().and_then(|mut child| child.wait());
        if let Err(error) = result {
            eprintln!("error: failed to invoke `lipo`: {error}");
            process::exit(1);
        }
    }
}

fn build(cmd: &Bundle, target: Option<&str>, packages: &[PackageInfo]) {
    let cargo_path =
        env::var("CARGO").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("cargo"));
    let mut cargo = Command::new(cargo_path);
    cargo.arg("build");

    for package_info in packages {
        cargo.args(["--package", &package_info.package_name]);
    }

    cargo.arg("--lib");

    if cmd.release {
        cargo.arg("--release");
    }
    if let Some(profile) = &cmd.profile {
        cargo.args(["--profile", profile]);
    }

    if !cmd.features.is_empty() {
        cargo.arg("--features");
        for feature in &cmd.features {
            cargo.arg(feature);
        }
    }
    if cmd.all_features {
        cargo.arg("--all-features");
    }
    if cmd.no_default_features {
        cargo.arg("--no-default-features");
    }

    if let Some(target) = target {
        cargo.args(["--target", target]);
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
}

fn bundle_clap(package_info: &PackageInfo, out_dir: &Path, target: &Target) {
    let src = out_dir.join(&package_info.lib_name);

    let name = &package_info.name;
    let bundle_path = out_dir.join(format!("bundle/{name}.clap"));

    match target.os {
        Os::Linux | Os::Windows => {
            let dst = bundle_path;

            fs::create_dir_all(dst.parent().unwrap()).unwrap();
            fs::copy(&src, &dst).unwrap();
        }
        Os::MacOs => {
            if bundle_path.exists() {
                fs::remove_dir_all(&bundle_path).unwrap();
            }

            let dst = bundle_path.join(format!("Contents/MacOS/{name}"));

            fs::create_dir_all(dst.parent().unwrap()).unwrap();
            fs::copy(&src, &dst).unwrap();

            macos_bundle_info(package_info, &bundle_path);
        }
    }
}

fn bundle_vst3(package_info: &PackageInfo, out_dir: &Path, target: &Target) {
    let src = out_dir.join(&package_info.lib_name);

    let name = &package_info.name;
    let bundle_path = out_dir.join(format!("bundle/{name}.vst3"));

    let dst = match target.os {
        Os::Linux => {
            let arch_str = match target.arch {
                Arch::Aarch64 => "aarch64",
                Arch::I686 => "i386",
                Arch::X86_64 => "x86_64",
                Arch::Aarch64X86_64 => unreachable!(),
            };

            bundle_path.join(format!("Contents/{arch_str}-linux/{name}.so"))
        }
        Os::MacOs => bundle_path.join(format!("Contents/MacOS/{name}")),
        Os::Windows => {
            let arch_str = match target.arch {
                Arch::Aarch64 => "arm64",
                Arch::I686 => "x86",
                Arch::X86_64 => "x86_64",
                Arch::Aarch64X86_64 => unreachable!(),
            };

            bundle_path.join(format!("Contents/{arch_str}-win/{name}.vst3"))
        }
    };

    if bundle_path.exists() {
        fs::remove_dir_all(&bundle_path).unwrap();
    }

    fs::create_dir_all(dst.parent().unwrap()).unwrap();
    #[allow(clippy::needless_borrows_for_generic_args)]
    fs::copy(&src, &dst).unwrap();

    if target.os == Os::MacOs {
        macos_bundle_info(package_info, &bundle_path);
    }
}

fn macos_bundle_info(package_info: &PackageInfo, bundle_path: &Path) {
    let name = &package_info.name;

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>{name}</string>
    <key>CFBundleIdentifier</key>
    <string></string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
</dict>
</plist>"#
    );

    fs::write(bundle_path.join("Contents/Info.plist"), plist).unwrap();
    fs::write(bundle_path.join("Contents/PkgInfo"), "BNDL????").unwrap();
}
