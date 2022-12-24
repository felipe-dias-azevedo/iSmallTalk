use chrono::{DateTime, Local};
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(Eq, PartialEq, EnumString, Display)]
pub enum TypeSystem {
    AddClients,
}

#[derive(Eq, PartialEq)]
pub enum TypeChat {
    Error,
    Warning,
    Info,
    System(TypeSystem),
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

    pub fn from_type_system(type_system: TypeSystem) -> Self {
        ChatInfo {
            type_chat: TypeChat::System(type_system),
            text: String::new(),
            time: Local::now(),
        }
    }

    pub fn to_string(&self) -> String {
        let color = match &self.type_chat {
            TypeChat::Error => String::from("#df0e0f"),
            TypeChat::Warning => String::from("#f7c02d"),
            TypeChat::Info => String::from("#7bafe9"),
            _ => String::from(""),
        };

        // if let TypeChat::System(type_system) = &self.type_chat {
        //     return match type_system {
        //         TypeSystem::AddClients => TypeSystem::AddClients.to_string()
        //     }
        // }

        format!(
            "<span color='{}'>{} : {}<span>\n",
            color,
            self.time.format("%d/%m/%Y %T"),
            self.text.to_owned()
        )
    }
}
