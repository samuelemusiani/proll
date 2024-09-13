use chrono::{DateTime, Utc};
use core::panic;
use std::{
    fmt, fs,
    io::{Read, Write},
    path::Path,
    str::FromStr,
    u16,
};

use minreq;
use xz::read::XzDecoder;

const ARCH_URL: &str = "https://archive.archlinux.org";
const INDEX_PATH: &str = "/packages/.all/index.0.xz";
const PKG_POSTFIX1: &str = ".pkg.tar.zst";
const PKG_POSTFIX2: &str = ".pkg.tar.xz";
const CACHE_DURATION: i64 = 5; // 5 minutes

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

pub fn get_pkg_index() -> Result<String> {
    let pkg_list = read_cache();
    match pkg_list {
        Ok(s) => match s {
            Some(s) => return Ok(s),
            None => {}
        },
        Err(e) => eprintln!("Error: Reading cache: {e}"),
    }

    let res = minreq::get(ARCH_URL.to_owned() + INDEX_PATH).send()?;
    let cursor = std::io::Cursor::new(res.into_bytes());

    // The 34M value is derived by printing the capacity of the string when full
    let mut pkg_list = String::with_capacity(34_000_000);
    XzDecoder::new(cursor).read_to_string(&mut pkg_list)?;

    write_cache(&pkg_list)?;

    Ok(pkg_list)
}

fn read_cache() -> Result<Option<String>> {
    let p = Path::new("/tmp/.proll");

    if !p.exists() {
        return Ok(None);
    }

    let last_write = fs::read_to_string(p.join("date"))?;
    let last_write: DateTime<Utc> = DateTime::from_str(&last_write)?;
    let minutes = (Utc::now() - last_write).num_minutes();
    if minutes > CACHE_DURATION {
        return Ok(None);
    }

    let pkgs = fs::read_to_string(p.join("pkgs"))?;
    Ok(Some(pkgs))
}

fn write_cache(pkgs: &str) -> Result<()> {
    let p = Path::new("/tmp/.proll");
    if !p.exists() {
        fs::create_dir(p)?;
    };

    let mut pkgs_file = fs::File::create(p.join("pkgs"))?;
    pkgs_file.write_all(pkgs.as_bytes())?;

    let last_write = Utc::now();
    let mut date_file = fs::File::create(p.join("date"))?;
    date_file.write_all(&last_write.to_string().as_bytes())?;

    Ok(())
}

#[derive(Debug)]
pub enum Arch {
    X86_64,
    Any,
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Arch::X86_64 => write!(f, "x86_64"),
            Arch::Any => write!(f, "any"),
        }
    }
}

#[derive(Debug)]
pub struct Package {
    name: String,
    version: String,
    build_version: u16,
    arch: Arch,
}

impl Package {
    pub fn parse(package: &str) -> Result<Package> {
        let x86_str = "x86_64";
        let any_str = "any";

        let index = package
            .rfind('-')
            .ok_or("Cannot find first '-' in package str")?;

        let (package, arch) = package.split_at(index);
        let arch_str = (arch.strip_prefix('-')).ok_or("Parse error. Cannot strip - from arch")?;
        let arch = if arch_str == x86_str {
            Arch::X86_64
        } else if arch_str == any_str {
            Arch::Any
        } else {
            return Err(String::from("Parse error. Cannot find arch").into());
        };

        let index = package
            .rfind('-')
            .ok_or("Cannot find second '-' in package str")?;

        let (package, build_version) = package.split_at(index);
        let build_version: u16 = (build_version.strip_prefix('-'))
            .ok_or("Parse error. Cannot strip - from build_version")?
            .parse()?;

        let index = package
            .rfind('-')
            .ok_or("Cannot find third '-' in package str")?;

        let (package, version) = package.split_at(index);
        let version = (version.strip_prefix('-'))
            .ok_or("Parse error. Cannot strip - from build_version")?
            .to_owned();

        let name = package.to_owned();

        Ok(Package {
            name,
            version,
            build_version,
            arch,
        })
    }

    pub fn full_name(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.name, self.version, self.build_version, self.arch
        )
    }

    /// The postfix can be of two types, so we return a tuple of
    /// urls and the caller check which is correct
    pub fn get_url(&self) -> Result<String> {
        let mut base = format!(
            "{ARCH_URL}/packages/{}/{}",
            self.name.chars().next().unwrap(),
            self.name(),
        );

        let name1 = format!("{}{}", self.full_name(), PKG_POSTFIX1);
        let name2 = format!("{}{}", self.full_name(), PKG_POSTFIX2);

        let res = match minreq::get(&base).send() {
            Ok(res) => res,
            Err(e) => panic!("Getting base packages: {e}"),
        };

        let res = res.as_str()?;

        if res.contains(&name1) {
            base.push('/');
            base.push_str(&name1);
            Ok(base)
        } else if res.contains(&name2) {
            base.push('/');
            base.push_str(&name2);
            Ok(base)
        } else {
            Err("Could not find package extension".into())
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn build_version(&self) -> u16 {
        self.build_version
    }

    pub fn arch(&self) -> &Arch {
        &self.arch
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_packages() {
        let pkg = Package::parse("caddy-2.4.3-1-x86_64").unwrap();
        assert_eq!(&pkg.name, "caddy");
        assert_eq!(&pkg.version, "2.4.3");
        assert_eq!(pkg.build_version, 1);
        assert!(matches!(pkg.arch, Arch::X86_64));

        let pkg = Package::parse("foo-bar-7.1.0-18-any").unwrap();
        assert_eq!(&pkg.name, "foo-bar");
        assert_eq!(&pkg.version, "7.1.0");
        assert_eq!(pkg.build_version, 18);
        assert!(matches!(pkg.arch, Arch::Any));
    }
}
