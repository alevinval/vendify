use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
/// CLI tool to vendor and manage dependencies.
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// Enable debug logging
    #[clap(short, long, takes_value = false, parse(from_flag))]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize the working directory
    Init {},

    /// Add a dependency to the spec file
    Add {
        /// Git URL of the repository to vendor
        url: String,

        /// A branch, commit or tag
        #[clap(default_value = "master")]
        refname: String,

        /// Extensions to vendor
        #[clap(short, long)]
        extensions: Option<Vec<String>>,

        /// Target paths that will be vendored
        #[clap(short, long)]
        targets: Option<Vec<String>>,

        /// Ignored paths that will NOT be vendored
        #[clap(short, long)]
        ignores: Option<Vec<String>>,
    },

    /// Vendors the dependencies respecting the lock pins
    Install {},

    /// Updates the vendored dependencies according to the desired refname
    /// in the spec file, updates the pins in the lock file.
    Update {},

    ClearCache {},
}
