use super::{chat_message::ChatMessage, chat_info::ChatInfo};

pub struct WindowInfo {
    pub chat_message: Option<ChatMessage>,
    pub chat_info: Option<ChatInfo>,
    pub is_sent: bool
}

impl WindowInfo {
    pub fn new_chat_message(chat_message: ChatMessage, is_sent: bool) -> Self {
        WindowInfo { chat_message: Some(chat_message), chat_info: None, is_sent }
    }

    pub fn new_chat_info(chat_info: ChatInfo, is_sent: bool) -> Self {
        WindowInfo { chat_message: None, chat_info: Some(chat_info), is_sent }
    }

    pub fn get_chat(&self) -> Option<String> {

        if let Some(chat_info) = &self.chat_info {
            return Some(chat_info.to_string());
        }

        if let Some(chat_message) = &self.chat_message {
            return Some(chat_message.to_string());
        }

        None
    }

    pub fn is_chat_info(&self) -> bool {
        self.chat_info.is_some()
    }

    pub fn is_chat_message(&self) -> bool {
        self.chat_message.is_some()
    }
}