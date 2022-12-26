use chrono::{DateTime, Local};

#[derive(Eq, PartialEq)]
pub enum TypeChat {
    Error,
    Warning,
    Info,
}

pub struct ChatInfo {
    pub type_chat: TypeChat,
    pub text: String,
    pub time: DateTime<Local>,
}

impl ChatInfo {
    pub fn new(type_chat: TypeChat, text: String) -> Self {
        ChatInfo {
            type_chat,
            text,
            time: Local::now(),
        }
    }

    pub fn to_send_text(&self) -> String {
        format!(
            "CI {} : {}<span>",
            self.time.format("%d/%m/%Y %T"),
            self.text.to_owned()
        )
    }

    pub fn to_chat_text(&self) -> String {
        let color = match &self.type_chat {
            TypeChat::Error => String::from("#df0e0f"),
            TypeChat::Warning => String::from("#f7c02d"),
            TypeChat::Info => String::from("#7bafe9")
        };

        format!(
            "<span color='{}'>{} : {}<span>\n",
            color,
            self.time.format("%d/%m/%Y %T"),
            self.text.to_owned()
        )
    }
}
