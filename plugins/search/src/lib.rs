extern crate hyper;
extern crate scraper;
extern crate url;

use std::error::Error;
use std::io::prelude::*;

pub fn query_google(query: &str) -> Result<String, Box<Error>> {
    let client = hyper::Client::new();

    let msg = format!("http://www.google.com/search?q={}", query);

    let mut res = client.get(&msg).send()?;
    if res.status != hyper::Ok {
        return Err("Something went wrong with the request".into());
    }
    let mut body = Vec::new();
    res.read_to_end(&mut body)?;
    Ok(String::from_utf8_lossy(&body).into_owned())
}

pub fn parse_first_result(body: &str) -> Result<String, Box<Error>> {
    use scraper::{Html, Selector};

    let html = Html::parse_document(&body);
    let sel = Selector::parse("cite").unwrap();
    let cite = html.select(&sel).next().ok_or("There should be a <cite>, but there isn't")?;
    Ok(cite.text().next().ok_or("Wat. <cite> has no text.")?.to_owned())
}

#[test]
#[ignore]
fn test_parse_first_result_on_dump() {
    use std::fs::File;
    let mut f = File::open("../../dump.txt").unwrap();
    let mut body = String::new();
    f.read_to_string(&mut body).unwrap();
    println!("{:?}", parse_first_result(&body));
}

#[no_mangle]
pub fn respond_to_command(cmd: &str, _sender: &str) -> String {
    if cmd == "search" {
        return "You need to search for something, retard.".into();
    }
    if cmd.starts_with("search ") {
        let wot = cmd[7..].trim();
        if wot.is_empty() {
            return "Empty search? Impossible!".into();
        }
        match query_google(wot) {
            Ok(body) => {
                match parse_first_result(&body) {
                    Ok(result) => return result,
                    Err(e) => {
                        use std::fs::File;

                        let path = "dump.txt";
                        let mut file = File::create(path).unwrap();
                        file.write_all(body.as_bytes()).unwrap();
                        println!("CORE DUMPED ({})", path);
                        return format!("Error: {}", e);
                    }
                }
            }
            Err(e) => return format!("Error when googuring: {}", e),
        }
    }
    return "".into();
}
