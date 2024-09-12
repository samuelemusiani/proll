use clap::{Parser, Subcommand};
use colored::Colorize;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

/// Make pacman package rollbacks easy
#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List version of a package
    #[command(arg_required_else_help = true)]
    List {
        /// Name of the package
        package: String,
    },
}

pub fn list(package: &str) -> Result<String> {
    let pkg_list = proll::get_pkg_index()?;

    let colored_package = package.red();

    let mut result = String::new();

    for pkg in pkg_list.split('\n') {
        if pkg.contains(package) {
            let mut splits = pkg.split(package);
            result.push_str(&format!("{}", splits.next().unwrap()));
            for spl in splits {
                result.push_str(&format!("{colored_package}{spl}"));
            }
            result.push('\n');
        }
    }

    Ok(result)
}
