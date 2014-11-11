#![feature(slicing_syntax, macro_rules)]
#![feature(overloaded_calls)]

extern crate http;

use std::os;
use std::str;
use std::sync::{Arc, Mutex};
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::collections::HashMap;

use http::method::{Get, Post};
use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::{AbsolutePath};

type Scores = HashMap<String, i32>;

#[deriving(Send,Clone)]
struct KarmaServer {
    scores: Arc<Mutex<Scores>>,
}

impl KarmaServer {
    fn new(scores: Scores) -> KarmaServer {
        KarmaServer {
            scores: Arc::new(Mutex::new(scores)),
        }
    }
}

#[deriving(Show)]
struct SlackPayload<'a> {
    token: &'a str,
    team_id: &'a str,
    channel_id: &'a str,
    channel_name: &'a str,
    user_id: &'a str,
    user_name: &'a str,
    command: &'a str,
    text: &'a str,
}

macro_rules! get(
    ($hash: expr, $opt:expr, $msg:expr) => {
        match $hash.get(&$opt) {
            Some(n) => *n,
            None => return Err($msg.to_string()),
        }
    }
)

impl<'a> SlackPayload<'a> {
    fn from_body<'a>(req: &[u8]) -> Result<SlackPayload, String> {
        let pieces = req.split(|c| *c == '&' as u8);
        let pieces: Vec<&str> = pieces.map(|c| str::from_utf8(c).unwrap()).collect();
        let mut vars = HashMap::new();

        for c in pieces.iter() {
            let mut l1 = c.splitn(1, '=');
            match (l1.next(), l1.next()) {
                (Some(m1), Some(m2)) => { vars.insert(m1, m2); },
                _ => {},
            }
        }

        return Ok(SlackPayload {
            token:          get!(vars, "token", "No `token` in request"),
            team_id:        get!(vars, "team_id", "No `team_id` in request"),
            channel_id:     get!(vars, "channel_id", "No `channel_id` in request"),
            channel_name:   get!(vars, "channel_name", "No `channel_name` in request"),
            user_id:        get!(vars, "user_id", "No `user_id` in request"),
            user_name:      get!(vars, "user_name", "No `user_name` in request"),
            command:        get!(vars, "command", "No `command` in request"),
            text:           get!(vars, "text", "No `text` in request"),
        })
    }
}

fn handle_karma(req: Vec<u8>, points: &mut Scores) {
    let payload = match SlackPayload::from_body(req.as_slice()) {
        Ok(payload) => payload,
        Err(err) => {
            println!("Error! {}", err); return
        }
    };

    let op = match payload.command {
        "%2F++" => |c: i32| c + 1,
        "%2F--" => |c: i32| c - 1,
        _ => return,
    };

    let current = points.insert(payload.user_name.to_string(), 0).unwrap_or(0);
    points.insert(payload.user_name.to_string(), op(current));

    println!("{} now has {}", payload.user_name, op(current));
}

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
            (&Post, "/slack") => {
                let mut scores = (*self.scores).lock();
                handle_karma(r.body, &mut *scores);
            },
            (_, _) => {
                w.write(b"Not found :(");
            }
        }
    }
}

fn main() {
    let server = KarmaServer::new(HashMap::new());
    server.serve_forever();
}
