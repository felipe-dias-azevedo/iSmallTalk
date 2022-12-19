use chrono::{DateTime, Local};

pub struct Message {
    pub id: u32,
    pub text: String,
    pub time: DateTime<Local>
}

impl Message {
    pub fn new(text: String) -> Self {
        Message { id: rand::random::<u32>(), text, time: Local::now() }
    }

    pub fn show(&self) -> String {
        format!("{} - {} : {}", 
            self.id,
            self.time.format("%d/%m/%Y %T"),
            self.text.to_owned())
    }
}