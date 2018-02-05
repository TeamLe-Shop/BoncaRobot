extern crate reqwest;

use std::error::Error;
use std::io::Read;

pub fn fetch_string(link: &str) -> Result<String, Box<Error>> {
    let mut resp = reqwest::get(link)?;

    let status = resp.status();

    if !status.is_success() {
        return Err(status.to_string().into());
    }

    let mut content = Vec::new();
    resp.read_to_end(&mut content)?;
    Ok(String::from_utf8_lossy(&content).into_owned())
}
