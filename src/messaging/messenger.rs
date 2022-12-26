use std::io::{Error, Write};
use std::net::{IpAddr, TcpStream};

pub struct Messenger {
    pub ip: IpAddr,
    pub port: u16,
    pub client: Option<TcpStream>,
}

impl Messenger {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Messenger {
            ip,
            port,
            client: None,
        }
    }

    pub fn from(ip: IpAddr, port: u16, client: TcpStream) -> Self {
        Messenger {
            ip,
            port,
            client: Some(client),
        }
    }

    fn add_connection(&mut self, connection: TcpStream) {
        self.client = Some(connection);
    }

    pub fn send_message(&mut self, text: &String) -> Option<Error> {
        if self.client.is_none() {
            return None;
        }

        let mut tcp = self.client.as_ref().unwrap();

        let message_sent = tcp.write(text.as_bytes());

        let received_error = message_sent.err();

        if received_error.is_some() {
            self.client = None;
        }

        received_error
    }

    pub fn get_id(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
