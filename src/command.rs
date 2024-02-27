use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name="plugin-cli", version, about, long_about = None)]
pub enum Cli {
    /// Initialize code for a plugin
    Create {
        /// The name of the plugin, in kebab-case
        name: Option<String>,
        /// The url to the template repository (optional) or the path to the template (optional)
        #[arg(short, long)]
        template: Option<String>,
        /// Is the template option a git remote? (default: true)
        #[arg(short, long)]
        git_template: Option<bool>,
    },
    /// Package the plugin into a distributable package
    Package {
        /// The path to the plugin project
        path: Option<PathBuf>,
    },
    // TODO
    Publish,
}
