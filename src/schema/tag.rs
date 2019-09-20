#[derive(Clone, Debug)]
pub struct Tag {
    text: String,
}

impl Tag {
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}
