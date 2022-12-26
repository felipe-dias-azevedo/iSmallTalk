use crate::channel::chat_info::ChatInfo;
use crate::channel::chat_message::ChatMessage;
use std::net::{IpAddr, TcpStream};

pub enum SystemAction {
    LeaveChat,
    LeaveChatAndQuit,
    ClientExitChat(String),
    SendChatMessage(ChatMessage, bool),
    SendChatInfo(ChatInfo),
    ToggleHost(bool),
    RequestAddClient(IpAddr, u16, TcpStream),
    AddClient(IpAddr, u16, TcpStream),
    ResetMainTextEntry
}
