use crate::channel::chat_info::{ChatInfo, TypeChat};
use crate::channel::chat_message::ChatMessage;
use crate::channel::system_action::SystemAction;
use crate::networking::client;
use gtk::glib::{clone, Sender};
use std::io::Read;
use std::net::TcpListener;
use std::thread;
use std::thread::JoinHandle;

pub fn start(id: &String, sender: &Sender<SystemAction>, server: TcpListener) -> JoinHandle<()> {
    let id = id.to_owned();

    thread::spawn(clone!(
        @strong id,
        @strong sender,
        => move || {
            for client in server.incoming() {
                if client.is_err() {
                    let error = client.as_ref().unwrap_err().to_string();
                    println!("ERROR: {}", error);
                    sender
                        .send(SystemAction::SendChatInfo(
                            ChatInfo::new(
                                String::from(&id),
                                TypeChat::Error,
                                error,
                            ),
                            false,
                        ))
                        .unwrap();
                }

                thread::spawn(clone!(
                    @strong id,
                    @strong sender,
                    => move || loop {
                        let mut stream = client.as_ref().unwrap();

                        let mut buffer = [0; 1024];

                        stream
                            .read(&mut buffer)
                            .expect("Couldn't read from buffer...");

                        let text = String::from_utf8(Vec::from(&buffer[..])).unwrap();

                        let text = text.trim_end_matches(char::from(0));

                        if text.is_empty() {
                            break;
                        }

                        if text.starts_with("CM") {
                            let text = text.split("CM ").last().unwrap_or("").to_string();
                            sender
                                .send(SystemAction::SendChatMessage(
                                    ChatMessage::from(text),
                                    false,
                                ))
                                .unwrap();
                        } else if text.starts_with("CI") {
                            let text = text.split("CI ").last().unwrap_or("").to_string();
                            sender
                                .send(SystemAction::SendChatInfo(
                                    ChatInfo::from(String::from(&id), text),
                                    false,
                                ))
                                .unwrap();
                        } else if text.starts_with("RAC") {
                            let ip_port = text.split("RAC ").last().unwrap_or("").to_string();

                            let tcp_stream = client::connect(&ip_port);
                            let (ip, port) = client::id_to_ip_port(&ip_port);

                            if let Some(client) = tcp_stream {
                                sender
                                    .send(SystemAction::AddClient(ip, port, client))
                                    .unwrap();
                                // TODO: Cancel request add client if error

                                sender
                                    .send(SystemAction::SendChatInfo(
                                        ChatInfo::new(
                                            String::from(&ip_port),
                                            TypeChat::Info,
                                            format!("User {} connected the chat", ip_port),
                                        ),
                                        false,
                                    ))
                                    .unwrap();
                            }
                        } else if text.starts_with("CEC") {
                            let ip_port = text.split("CEC ").last().unwrap_or("").to_string();

                            sender
                                .send(SystemAction::ClientExitChat(String::from(&ip_port)))
                                .unwrap();

                            sender
                                .send(SystemAction::SendChatInfo(
                                    ChatInfo::new(
                                        String::from(&ip_port),
                                        TypeChat::Info,
                                        format!("User {} left the chat", ip_port),
                                    ),
                                    false,
                                ))
                                .unwrap();
                        }
                }));
            }
    }))
}
