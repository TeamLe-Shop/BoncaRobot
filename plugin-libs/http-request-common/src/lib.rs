extern crate reqwest;

use std::error::Error;
use std::io::Read;

pub fn fetch_string(link: &str) -> Result<String, Box<Error>> {
    let mut resp = reqwest::get(link)?;

    if !resp.status().is_success() {
        return Err("Something went wrong with the request".into());
    }

    let mut content = Vec::new();
    resp.read_to_end(&mut content)?;
    Ok(String::from_utf8_lossy(&content).into_owned())
}
