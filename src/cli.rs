use clap::{Parser, Subcommand};
use colored::Colorize;
use proll::Package;

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

    let mut result = String::new();
    result.push_str(&format!(
        "{}\t\t{}\t\t{}\t{}\n",
        "Name", "Version", "Build", "Arch"
    ));

    for pkg in pkg_list.split('\n') {
        if pkg.contains(package) {
            let p = Package::parse(pkg)?;
            result.push_str(&format!(
                "{}\t\t{}\t\t{}\t{}\n",
                color_match(p.name(), package),
                p.version(),
                p.build_version(),
                p.arch()
            ));
        }
    }

    Ok(result)
}

/// Return s with the substrings equal to m colored
fn color_match(s: &str, m: &str) -> String {
    let mut result = String::with_capacity(s.len());

    let mut splits = s.split(m);

    result.push_str(&format!("{}", splits.next().unwrap()));
    for spl in splits {
        result.push_str(&format!("{}{spl}", m.red()));
    }

    result
}
