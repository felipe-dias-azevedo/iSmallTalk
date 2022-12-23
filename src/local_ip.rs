use std::net::{UdpSocket, IpAddr, Ipv4Addr};

pub fn get_optional() -> Option<IpAddr> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None,
    };

    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None,
    };

    match socket.local_addr() {
        Ok(addr) => return Some(addr.ip()),
        Err(_) => return None,
    };
}

pub fn get() -> IpAddr {
    let ip = get_optional();

    if ip.is_some() {
        return ip.unwrap()
    }

    IpAddr::V4(Ipv4Addr::LOCALHOST)
}