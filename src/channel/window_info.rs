use super::{chat_info::ChatInfo, chat_message::ChatMessage};

pub enum WindowInfo {
    ChatMessage(ChatMessage, bool),
    ChatInfo(ChatInfo, bool),
}

impl WindowInfo {
    pub fn get_send_text(&self) -> String {
        match self {
            WindowInfo::ChatMessage(m, is_sent) => m.to_send_text(),
            WindowInfo::ChatInfo(m, is_sent) => m.to_chat_text(),
        }
    }

    pub fn get_chat_data(&self) -> (String, &bool) {
        match self {
            WindowInfo::ChatMessage(m, is_sent) => (m.to_chat_text(), is_sent),
            WindowInfo::ChatInfo(m, is_sent) => (m.to_chat_text(), is_sent),
        }
    }
}
