#![feature(slicing_syntax)]

extern crate http;

use std::os;
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use http::method::{Get, Post};
use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::{AbsolutePath};

#[deriving(Clone)]
struct KarmaServer;

impl Server for KarmaServer {
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
        let path = match r.request_uri { AbsolutePath(ref path) => path[], _ => return };

        println!("{}: {}", r.method, path);

        match (&r.method, path) {
            (&Get, "/") => {
                w.write(b"Hello world!");
            },
            (&Post, "/command") => {
                w.write(b"Hit Command");
            },
            (_, _) => {
                w.write(b"Not found :(");
            }
        }
    }
}

fn main() {
    KarmaServer.serve_forever();
}
