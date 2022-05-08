use clap::{AppSettings, Args, Parser, Subcommand};

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
    let Cargo::Coupler(coupler) = Cargo::parse();

    match coupler {
        Coupler::Bundle(bundle) => {

        }
    }
}
