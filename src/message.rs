use std::vec;

use crate::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Messages {
    messages: Vec<(String, SerializableColour)>,
}

impl Messages {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    /// Add new message as a tuple
    pub fn add<T: Into<String>>(&mut self, message: T, colour: SerializableColour) {
        self.messages.push((message.into(), colour));
    }


    /// Create `DoubleEndedIterator` over messages
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &(String, SerializableColour)> {
        self.messages.iter()
    }
}

pub fn msgbox(text: &str, width: i32, root: &mut Root) {
    let options: &[&str] = &[];
    menu(text, options, width, root);
}