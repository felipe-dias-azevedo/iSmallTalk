use std::io::{Error, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use crate::networking::local_ip;
use crate::networking::server;

#[derive(PartialEq)]
pub enum TypeMessenger {
    Host,
    Client,
}

pub struct Messenger {
    pub type_messenger: TypeMessenger,
    pub ip: IpAddr,
    pub port: u16,
    //pub server: TcpListener,
    //pub server: Arc<Mutex<TcpListener>>,
    pub clients: Vec<TcpStream>,
}

impl Messenger {
    fn get_type_messenger(is_host: bool) -> TypeMessenger {
        if is_host {
            TypeMessenger::Host
        } else {
            TypeMessenger::Client
        }
    }

    pub fn new(is_host: bool) -> (Self, Arc<Mutex<TcpListener>>) {
        let type_messenger = Messenger::get_type_messenger(is_host);
        let ip = local_ip::get();
        let (server, port) = server::bind_ip_port(&ip);

        (
            Messenger {
                type_messenger,
                ip,
                port,
                // server: Arc::new(Mutex::new(server)),
                clients: Vec::new(),
            },
            Arc::new(Mutex::new(server)),
        )
    }

    pub fn change_type(&mut self, is_host: bool) {
        self.type_messenger = Messenger::get_type_messenger(is_host);
    }

    pub fn get_chat_history(&self) -> Option<String> {
        if self.type_messenger == TypeMessenger::Client {
            return None;
        }

        None
    }

    pub fn connect() {}

    pub fn add_client() {}

    pub fn get_id(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn send_text(&mut self, text: String) -> Vec<Error> {
        let mut sends = vec![];

        for i in 0..self.clients.len() {
            let sent_result = self.clients[i].write(text.as_bytes());

            if sent_result.is_err() {
                sends.push((i, sent_result.unwrap_err()));
            }
        }

        let mut errors = vec![];

        for (corrupted_index, err) in sends {
            self.clients.remove(corrupted_index);
            errors.push(err);
        }

        errors
    }

    // fn send(&mut self, text: String) -> Vec<Error> {
    //     self.clients
    //         .iter()
    //         .map(|mut client| client.write(text.as_bytes()))
    //         .filter_map(|e| e.err())
    //         .collect()
    // }
}
