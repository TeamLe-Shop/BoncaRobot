extern crate percent_encoding;
extern crate reqwest;

use percent_encoding::{utf8_percent_encode, QUERY_ENCODE_SET};
use std::error::Error;
use std::io::Read;
use reqwest::StatusCode;

pub fn fetch_string(base: &str, user_query: &str) -> Result<(String, StatusCode), Box<Error>> {
    // "Tame" the user query, percent-encoding '%' and '/'.
    // Apparently, percent_encoding doesn't even encode '&'. Jeezus.
    let tamed_user_query = user_query
        .replace('%', "%25")
        .replace('/', "%2F")
        .replace('&', "%26");
    let encoded_user_query: String =
        utf8_percent_encode(&tamed_user_query, QUERY_ENCODE_SET).collect();
    let mut resp = reqwest::get(&format!("{}{}", base, encoded_user_query))?;
    let mut content = Vec::new();
    resp.read_to_end(&mut content)?;
    Ok((String::from_utf8_lossy(&content).into_owned(), resp.status()))
}
