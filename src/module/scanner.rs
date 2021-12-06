use std::net::{ SocketAddr, ToSocketAddrs };
use std::{ net::TcpStream, time::Duration };
use rayon::prelude::*;
use crate::commons::contants::{ TOP_100_PORTS };

#[derive(Debug, Clone)]
pub struct Port {
    pub port:  u16,
    pub state: bool,
}

#[derive(Debug, Clone)]
pub struct Domain {
    pub domain: String,
    pub open_port: Vec<Port>,
}

pub fn scan_ports(mut domain: Domain) -> Domain {
    domain.open_port = TOP_100_PORTS
        .into_par_iter()
        .map(|port| scan_port(&domain.domain, *port))
        .filter(|port| port.state)
        .collect();

    domain
}

pub fn scan_port(host: &str, port: u16) -> Port {
    let timeout = Duration::from_secs(3);
    let socket_addr: Vec<SocketAddr> = format!("{}:{}", host, port)
        .to_socket_addrs()
        .expect("can't create socket address.")
        .collect();

    if socket_addr.len() == 0 {
        return Port{
            port: port,
            state: false,
        };
    }

    let status = if let Ok(_) = TcpStream::connect_timeout(&socket_addr[0], timeout) {
        true
    } else {
        false
    };

    Port{
        port: port,
        state: status,
    }
}
