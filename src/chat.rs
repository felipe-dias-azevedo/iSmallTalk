use crate::message::Message;

pub struct Chat {
    pub messages: Vec<Message>
}

impl Chat {
    pub fn new() -> Self {
        Chat { messages: vec![] }
    }

    pub fn add_message(&mut self, text: String) {
        let message = Message::new(text);
        self.messages.push(message);
    }

    pub fn show(&self) -> String {
        self.messages
            .iter()
            .map(|x| x.show())
            .collect::<Vec<String>>()
            .join("\n")
    }
}