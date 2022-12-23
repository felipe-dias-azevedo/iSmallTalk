use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

pub struct ChatMessage {
    pub id: String,
    pub text: String,
    pub time: DateTime<Local>,
}

impl ChatMessage {
    pub fn new(id: &String, text: String) -> Self {
        ChatMessage {
            id: id.clone(),
            text,
            time: Local::now(),
        }
    }

    pub fn from(chat_message: String) -> Self {
        let texts = chat_message.split(':').collect::<Vec<&str>>();
        let text = texts.get(4).unwrap_or(&"").trim().to_string();

        let texts = texts[..4].join(":");

        let metadata = texts.split('-').collect::<Vec<&str>>();

        let id = metadata.first().unwrap_or(&"").trim().to_string();

        let time_string = metadata.last().unwrap_or(&"").trim();
        let time_datetime =
            NaiveDateTime::parse_from_str(time_string, "%d/%m/%Y %T").unwrap_or_default();
        let time = Local.from_local_datetime(&time_datetime).unwrap();

        ChatMessage { id, text, time }
    }

    pub fn to_string(&self) -> String {
        format!(
            // "<i>{} - {}</i> : <b>{}</b>\n",
            "{} - {} : {}\n",
            self.id,
            self.time.format("%d/%m/%Y %T"),
            self.text.to_owned()
        )
    }
}
