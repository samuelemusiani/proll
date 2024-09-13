use clap::{Parser, Subcommand};
use colored::Colorize;
use proll::Package;
use std::os::unix::process::CommandExt;
use std::process;

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

    /// Downgrade a package by version or by name
    #[command(arg_required_else_help = true)]
    Downgrade {
        /// Name of the package
        package: String,

        /// Version of the package
        version: Option<String>,
    },
}

pub fn list(package: String, raw_print: bool, no_color: bool) -> Result<String> {
    let pkg_list = proll::get_pkg_index()?;

    let mut result = String::new();
    if !raw_print {
        result.push_str(&format!(
            "{}\t\t{}\t\t{}\t{}\n",
            "Name", "Version", "Build", "Arch"
        ));
    }

    for pkg in pkg_list.split('\n') {
        if pkg.contains(&package) {
            if !raw_print {
                let p = Package::parse(pkg)?;

                let name = if no_color {
                    p.name()
                } else {
                    &color_match(p.name(), &package)
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
                    &color_match(pkg, &package)
                };

                result.push_str(&pkg);
                result.push('\n');
            }
        }
    }

    Ok(result)
}

pub fn downgrade(package: String, version: Option<String>) -> Result<String> {
    match &version {
        Some(v) => {
            if !v.chars().all(|x| x == '.' || x.is_numeric()) {
                return Err("Version not valid, should be in the form: 1.2.3\n".into());
            }
        }
        None => {}
    };

    let pkg_list = proll::get_pkg_index()?;
    let mut pkg: Vec<&str> = pkg_list
        .split('\n')
        .filter(|x| x.contains(&package))
        .collect();

    if let Some(version) = version {
        pkg = pkg
            .into_iter()
            .filter(|x| {
                let p = Package::parse(x);
                if let Ok(p) = p {
                    p.version().contains(&version)
                } else {
                    false
                }
            })
            .collect();
    };

    if pkg.len() > 1 {
        let pkgs = pkg.join("\n");
        let msg = "More than one package match. Please be more specific. Matched:\n";
        let mut err = String::with_capacity(msg.len() + pkgs.len());
        err.push_str(msg);
        err.push_str(&pkgs);
        err.push('\n');

        return Err(err.into());
    } else if pkg.len() == 0 {
        return Err("No package matched pattern\n".into());
    }

    println!("Matched package {}. Downgrading...", pkg[0].green());

    // Should check if present in local cache
    let p = Package::parse(pkg[0])?;
    let url = p.get_url()?;

    let err = process::Command::new("pacman").args(["-U", &url]).exec();

    Err(err.into())
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
