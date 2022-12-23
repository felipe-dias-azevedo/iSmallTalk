use std::net::{TcpListener, IpAddr};

const MIN_IP_PORT: u16 = 61202;
const MAX_IP_PORT: u16 = 61222;

pub fn bind_ip_port(ip: &IpAddr) -> (TcpListener, u16) {

    let mut port_attempt = MIN_IP_PORT;
    let mut port = 0u16;
    let mut server: Option<TcpListener> = None;

    while port == 0 {

        let addr = format!("{}:{}", ip.to_string(), port_attempt);

        let listener = TcpListener::bind(addr);

        if listener.is_err() && port_attempt < MAX_IP_PORT {
            port_attempt += 1;
            println!("ERROR: Binding IP address failed at {}:{}...", ip.to_string(), port_attempt);
            continue
        }

        if port_attempt >= MAX_IP_PORT {
            // TODO: return an option and show to the user that everything went wrong.
            panic!("Couldn't set server port between 61202 and 61220...");
        }

        port = port_attempt;
        server = listener.ok();
    }
    
    println!("server listening on {}:{}...", ip, port);
    return (server.unwrap(), port)
}