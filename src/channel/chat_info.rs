use std::str::FromStr;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use strum_macros::{Display, EnumString};

#[derive(Eq, PartialEq, EnumString, Display)]
pub enum TypeChat {
    Error,
    Warning,
    Info,
}

pub struct ChatInfo {
    pub id: String,
    pub type_chat: TypeChat,
    pub text: String,
    pub time: DateTime<Local>,
}

impl ChatInfo {
    pub fn new(id: String, type_chat: TypeChat, text: String) -> Self {
        ChatInfo {
            id,
            type_chat,
            text,
            time: Local::now(),
        }
    }

    pub fn from(id: String, chat_message: String) -> Self {
        let texts = chat_message.split(':').collect::<Vec<&str>>();

        let text = texts[3..].join(":").trim().to_string();

        let texts = texts[..3].join(":");

        let metadata = texts.split('-').collect::<Vec<&str>>();

        let type_chat = metadata.first().unwrap_or(&"").trim();
        let type_chat = TypeChat::from_str(type_chat).unwrap_or(TypeChat::Info);

        let time_string = metadata.last().unwrap_or(&"").trim();
        let time_datetime =
            NaiveDateTime::parse_from_str(time_string, "%d/%m/%Y %T").unwrap_or_default();
        let time = Local.from_local_datetime(&time_datetime).unwrap();

        ChatInfo { id, type_chat, text, time }
    }

    pub fn to_send_text(&self) -> String {
        format!(
            "CI {} - {} : {}",
            self.type_chat,
            self.time.format("%d/%m/%Y %T"),
            self.text.to_owned()
        )
    }

    pub fn to_chat_text(&self) -> String {
        let color = match &self.type_chat {
            TypeChat::Error => String::from("#df0e0f"),
            TypeChat::Warning => String::from("#f7c02d"),
            TypeChat::Info => String::from("#7bafe9"),
        };

        format!(
            "<span color='{}'>{} : {}</span>\n",
            color,
            self.time.format("%d/%m/%Y %T"),
            self.text.to_owned()
        )
    }
}
