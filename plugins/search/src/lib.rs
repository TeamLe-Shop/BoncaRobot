extern crate hyper;
extern crate scraper;
extern crate url;

use hyper::Client;
use scraper::{Html, Selector};
use std::error::Error;
use std::io::prelude::*;

pub fn do_search(query: &str) -> Result<String, Box<Error>> {
    let client = Client::new();

    let msg = format!("http://www.google.com/search?q={}", query);

    let mut res = client.get(&msg).send()?;
    if res.status != hyper::Ok {
        return Err("Something went wrong with the request".into());
    }
    let mut body = Vec::new();
    res.read_to_end(&mut body)?;
    let utf8 = String::from_utf8_lossy(&body).into_owned();
    let html = Html::parse_document(&utf8);
    let sel = Selector::parse("h3").map_err(|()| "Couldn't find h3 selectors or some shit.")?;

    for element in html.select(&sel) {
        let a = element.select(&Selector::parse("a").map_err(|()| "Could not find <a>. Dunno.")?)
                       .next().ok_or("Could not find <a>. Dunno m8.")?;
        let link = a.value().attr("href").ok_or("No href in the <a>? What the fuck?")?;
        let begin = link.find("q=").ok_or("Need some q= shit. It's how it works, don't ask me.")?;
        let end = link.find("&sa=")
            .ok_or("This &sa= bullshit. Don't ask me what it is, but I expected it and couldn't \
                    find it.")?;
        let s = &link[begin + 2..end];
        let decoded = url::percent_encoding::percent_decode(s.as_bytes()).decode_utf8()?;
        if !decoded.starts_with("http") {
            return Err(format!("Doesn't start with http: {}. Nag SneakySnake to fix eet",
                               decoded)
                .into());
        }
        return Ok(decoded.into_owned());
    }
    Err("Could find any h3 paragraphs. Blame google for not providing a search API causing me to \
         parse HTML half-arsedly."
        .into())
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
        match do_search(wot) {
            Ok(result) => return result,
            Err(e) => return format!("Error: {}", e),
        }
    }
    return "".into();
}
