use regex::Regex;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;

type Result<T> = std::result::Result<T, ValidateIpError>;

#[derive(Debug)]
pub struct ValidateIpError(pub &'static str);

static IP_VALID: &str = "^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5]):[0-9]+$";

pub fn connect(ip_port: &String) -> Option<TcpStream> {
    let socket = ip_port.parse::<SocketAddr>().ok();

    if socket.is_none() {
        return None;
    }

    let client = TcpStream::connect_timeout(&socket.unwrap(), Duration::from_millis(800));

    client.ok()
}

pub fn id_to_ip_port(ip_port: &String) -> (IpAddr, u16) {
    let ip_port_split = ip_port.split(":").collect::<Vec<&str>>();
    let ip = ip_port_split
        .first()
        .unwrap()
        .to_string()
        .parse::<IpAddr>()
        .unwrap();
    let port = ip_port_split
        .last()
        .unwrap()
        .to_string()
        .parse::<u16>()
        .unwrap();

    (ip, port)
}

pub fn validate_ip(text: &str) -> Result<TcpStream> {
    if text.is_empty() {
        return Err(ValidateIpError("Username ID is empty!"));
    }

    let rgx = Regex::new(IP_VALID).unwrap();

    let ip_valid = rgx.is_match(text);

    if !ip_valid {
        return Err(ValidateIpError("Username ID is not valid!"));
    }

    let port = text.split(':').last().unwrap().parse::<u16>();

    if port.is_err() {
        return Err(ValidateIpError("Username ID is not valid!"));
    }

    let socket = text.parse::<SocketAddr>();

    if socket.is_err() {
        return Err(ValidateIpError("Couldn't connect to username!"));
    }

    let client = TcpStream::connect_timeout(&socket.unwrap(), Duration::from_millis(800));

    if client.is_err() {
        return Err(ValidateIpError("Couldn't connect to username!"));
    }

    return Ok(client.unwrap());
}
