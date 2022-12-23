use chrono::{DateTime, Local};

pub struct ChatMessage {
    pub id: u32,
    pub text: String,
    pub time: DateTime<Local>
}

impl ChatMessage {
    pub fn new(text: String) -> Self {
        ChatMessage { id: rand::random::<u32>(), text, time: Local::now() }
    }

    pub fn to_string(&self) -> String {
        format!("{} - {} : {}",
            self.id,
            self.time.format("%d/%m/%Y %T"),
            self.text.to_owned())
    }
}
