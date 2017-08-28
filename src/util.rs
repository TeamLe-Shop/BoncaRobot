pub struct SplitChunks<'a> {
    text: &'a str,
    size: usize,
}

impl<'a> SplitChunks<'a> {
    pub fn new(text: &'a str, size: usize) -> Self {
        Self { text, size }
    }
}

impl<'a> Iterator for SplitChunks<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let mut cursor = self.size;
        while !self.text.is_char_boundary(cursor) {
            cursor -= 1;
        }
        let chunk = &self.text[..cursor];
        if chunk.is_empty() {
            return None;
        }
        self.text = &self.text[cursor..];
        Some(chunk)
    }
}

#[test]
fn test_split_chunks() {
    let text = "I am a cool guy lol";
    let mut chunks = SplitChunks::new(text, 5);
    assert_eq!(chunks.next(), Some("I am "));
    assert_eq!(chunks.next(), Some("a coo"));
    assert_eq!(chunks.next(), Some("l guy"));
    assert_eq!(chunks.next(), Some(" lol"));
    assert_eq!(chunks.next(), None);
}
