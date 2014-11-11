extern crate http;

use std::os;
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use http::server::{Config, Server, Request, ResponseWriter};

#[deriving(Clone)]
struct PlusPlusServer;

impl Server for PlusPlusServer {
    fn get_config(&self) -> Config {
        match os::getenv("PORT") {
            None => panic!("Must specify PORT"),
            Some(port) => {
                let port = from_str::<u16>(port.as_slice()).expect("PORT must be an int");
                Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: port } }
            }
        }
    }

    fn handle_request(&self, r: Request, w: &mut ResponseWriter) {
        w.write(b"Hello world!");
    }
}

fn main() {
    PlusPlusServer.serve_forever();
}
