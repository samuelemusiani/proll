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

        /// Do print with struct, show raw package names
        #[arg(short, long, default_value_t = false)]
        raw: bool,

        /// Do not color matched string
        #[arg(short, long, default_value_t = false)]
        no_color: bool,
    },
}

pub fn list(package: &str, raw_print: bool, no_color: bool) -> Result<String> {
    let pkg_list = proll::get_pkg_index()?;

    let mut result = String::new();
    if !raw_print {
        result.push_str(&format!(
            "{}\t\t{}\t\t{}\t{}\n",
            "Name", "Version", "Build", "Arch"
        ));
    }

    for pkg in pkg_list.split('\n') {
        if pkg.contains(package) {
            if !raw_print {
                let p = Package::parse(pkg)?;

                let name = if no_color {
                    p.name()
                } else {
                    &color_match(p.name(), package)
                };

                result.push_str(&format!(
                    "{}\t\t{}\t\t{}\t{}\n",
                    name,
                    p.version(),
                    p.build_version(),
                    p.arch()
                ));
            } else {
                let pkg = if no_color {
                    pkg
                } else {
                    &color_match(pkg, package)
                };

                result.push_str(&pkg);
                result.push('\n');
            }
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
