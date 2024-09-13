use std::fmt;
use std::io::Read;

use minreq;
use xz::read::XzDecoder;

const ARCH_URL: &str = "https://archive.archlinux.org";
const INDEX_PATH: &str = "/packages/.all/index.0.xz";
const PKG_POSTFIX: &str = ".pkg.tar.zst";

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

pub fn get_pkg_index() -> Result<String> {
    let res = minreq::get(ARCH_URL.to_owned() + INDEX_PATH).send()?;
    let cursor = std::io::Cursor::new(res.into_bytes());

    // The 34M value is derived by printing the capacity of the string when full
    let mut pkg_list = String::with_capacity(34_000_000);
    XzDecoder::new(cursor).read_to_string(&mut pkg_list)?;
    Ok(pkg_list)
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
    build_version: u8,
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
        let build_version: u8 = (build_version.strip_prefix('-'))
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

    pub fn get_url(&self) -> String {
        format!(
            "{ARCH_URL}/packages/{}/{}{}",
            self.name.chars().next().unwrap(),
            self.full_name(),
            PKG_POSTFIX
        )
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn build_version(&self) -> u8 {
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
