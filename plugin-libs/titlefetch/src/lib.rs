extern crate http_request_common;
extern crate scraper;

fn find_title(body: &str) -> String {
    use scraper::{Html, Selector};

    let html = Html::parse_document(body);
    let sel = Selector::parse("title").unwrap();
    let mut titles = html.select(&sel);
    match titles.next() {
        Some(title) => title.text().collect(),
        None => String::new(),
    }
}

pub fn get_title(link: &str) -> String {
    let (page, status) = match http_request_common::fetch_string(link, "") {
        Ok(page) => page,
        Err(e) => return format!("[error: {}]", e),
    };
    let title = find_title(&page);
    if status.is_success() {
        title
    } else {
        format!("[{}] {}", status, title)
    }
}
