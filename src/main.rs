#![feature(slicing_syntax, macro_rules)]
#![feature(overloaded_calls)]

extern crate http;
extern crate url;
extern crate serialize;

use std::os;
use std::sync::{Arc, Mutex};
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::collections::HashMap;

use http::method::{Get, Post};
use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::{AbsolutePath};

use serialize::json;

use incoming::{SlackPayload};
use outgoing::{SlackEndpoint, OutgoingWebhook};

mod incoming;
mod outgoing;

type Scores = HashMap<String, i32>;

#[deriving(Send,Clone)]
struct KarmaServer {
    scores: Arc<Mutex<Scores>>,
    endpoint: String,
}

impl KarmaServer {
    fn new(scores: Scores, endpoint: String) -> KarmaServer {
        KarmaServer {
            scores: Arc::new(Mutex::new(scores)),
            endpoint: endpoint
        }
    }
}

fn handle_karma(req: Vec<u8>, points: &mut Scores, cb: |&&str, &i32|) {
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

    cb(&payload.user_name, &op(current));
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

    #[allow(unused_must_use)]
    fn handle_request(&self, r: Request, w: &mut ResponseWriter) {
        let path = match r.request_uri { AbsolutePath(ref path) => path[], _ => return };

        println!("{}: {}", r.method, path);

        match (&r.method, path) {
            (&Get, "/") => {
                w.write(b"Hello world!");
            },
            (&Get, "/karma") => {
                let scores = (*self.scores).lock();
                let json = json::encode(&*scores);
                w.write(json.as_bytes());
            }
            (&Post, "/normalise") => {
                let mut scores = (*self.scores).lock();
                for (_, v) in scores.iter_mut() {
                    let new = *v as f64;
                    *v = new.sqrt() as i32;
                }
            }
            (&Post, "/slack") => {
                let mut scores = (*self.scores).lock();
                let slack = self.get_slack_endpoint();
                handle_karma(r.body, &mut *scores, |u, s| {
                    let msg = format!("{} now at {}", u, s);
                    let payload = OutgoingWebhook {
                        text: msg.as_slice(),
                        channel: "#hax",
                        username: "karmabot",
                        icon_emoji: Some(":ghost:"),
                    };

                    slack.send(&payload);
                });
            },
            (_, _) => {
                w.write(b"Not found :(");
            }
        }
    }
}

impl KarmaServer {
    fn get_slack_endpoint(&self) -> SlackEndpoint {
        return SlackEndpoint {
            url: self.endpoint.clone()
        }
    }
}

fn main() {
    let endpoint = match os::getenv("WEBHOOK_ENDPOINT") {
        Some(e) => e,
        None => panic!("Must set WEBHOOK_ENDPOINT"),
    };
    let server = KarmaServer::new(HashMap::new(), endpoint);
    server.serve_forever();
}
