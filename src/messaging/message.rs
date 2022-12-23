use chrono::{DateTime, Local};

pub struct Message {
    pub id: String,
    pub text: String,
    pub time: DateTime<Local>,
}

impl Message {
    pub fn new(id: String, text: String) -> Self {
        Message {
            id,
            text,
            time: Local::now(),
        }
    }

    pub fn show(&self) -> String {
        format!(
            "{} - {} : {}",
            self.id,
            self.time.format("%d/%m/%Y %T"),
            self.text.to_owned()
        )
    }
}
