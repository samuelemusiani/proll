use std::io::Read;

use minreq;
use xz::read::XzDecoder;

const ARCH_URL: &str = "https://archive.archlinux.org/";
const INDEX_PATH: &str = "packages/.all/index.0.xz";

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

pub fn get_pkg_index() -> Result<String> {
    let res = minreq::get(ARCH_URL.to_owned() + INDEX_PATH).send()?;
    let cursor = std::io::Cursor::new(res.into_bytes());

    let mut pkg_list = String::new();
    XzDecoder::new(cursor).read_to_string(&mut pkg_list)?;
    Ok(pkg_list)
}
