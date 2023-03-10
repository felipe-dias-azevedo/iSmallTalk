use crate::messaging::messenger::Messenger;
use std::io::{Error, Write};
use std::net::{IpAddr, TcpStream};

pub struct HostMessenger {
    pub ip: IpAddr,
    pub port: u16,
    pub clients: Vec<Option<Messenger>>,
}

impl HostMessenger {
    pub fn from(messenger: &Messenger) -> Self {
        HostMessenger {
            ip: messenger.ip,
            port: messenger.port,
            clients: vec![],
        }
    }

    pub fn add_connection(&mut self, ip: IpAddr, port: u16, connection: TcpStream) {
        let messenger = Messenger::from(ip, port, connection);

        self.clients.push(Some(messenger));
    }

    pub fn remove_connection(&mut self, ip_port: String) {
        let index_option = self.clients.iter().position(|x| {
            if let Some(y) = x {
                return y.get_id() == ip_port;
            }
            false
        });

        if let Some(client_index) = index_option {
            self.clients.remove(client_index);
        }
    }

    pub fn send_message(&mut self, text: &String) -> Vec<Error> {
        let mut errors = vec![];

        for i in 0..self.clients.len() {
            let client_option = &self.clients[i].as_ref().unwrap();

            let client = &client_option.client;
            let mut client = client.as_ref().unwrap();

            let message_sent = client.write(text.as_bytes());

            if message_sent.is_err() {
                self.clients.remove(i);
                errors.push(message_sent.unwrap_err());
            }
        }

        errors
    }

    pub fn send_broadcast_message(&mut self, text: &String, id_filter: &String) -> Vec<Error> {
        let mut errors = vec![];

        for i in 0..self.clients.len() {
            let client_option = &self.clients[i].as_ref().unwrap();

            if &client_option.get_id() == id_filter {
                continue;
            }

            let client = &client_option.client;
            let mut client = client.as_ref().unwrap();

            let message_sent = client.write(text.as_bytes());

            if message_sent.is_err() {
                self.clients.remove(i);
                errors.push(message_sent.unwrap_err());
            }
        }

        errors
    }

    pub fn get_ammount_connected(&self) -> u8 {
        self.clients.iter().filter(|c| c.is_some()).count() as u8
    }

    pub fn get_id(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
