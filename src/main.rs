#![feature(slicing_syntax, macro_rules)]
#![feature(overloaded_calls)]

extern crate redis;
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
mod persistence;

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

fn handle_karma(req: Vec<u8>, points: &mut Scores, cb: |&&str, &i32, &&str|) {
    let payload = match SlackPayload::from_body(req.as_slice()) {
        Ok(payload) => payload,
        Err(err) => {
            println!("Error! {}", err); return
        }
    };

    let op = match payload.command {
        "%2F%2B%2B" => |c: i32| c + 1,
        "%2F++" => |c: i32| c + 1,
        "%2F%2D%2D" => |c: i32| c - 1,
        "%2F--" => |c: i32| c - 1,
        _ => return,
    };

    let mut matches = payload.text.splitn(1, ' ');
    let target = match (matches.next(), matches.next()) {
        (None, None) => return, // No message or user
        (Some(user), None) => user, // user, no message
        (Some(user), Some(_)) => user, // TODO deal with the message
        (None, Some(_)) => unreachable!(),
    };

    let current = points.insert(target.to_string(), 0).unwrap_or(0);
    points.insert(target.to_string(), op(current));

    cb(&target, &op(current), &payload.channel_name);
}

impl Server for KarmaServer {
    fn get_config(&self) -> Config {
        match os::getenv("PORT") {
            None => panic!("Must specify PORT"),
            Some(port) => {
                let port = from_str::<u16>(port.as_slice()).expect("PORT must be an int");
                Config { bind_address: SocketAddr { ip: Ipv4Addr(0, 0, 0, 0), port: port } }
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
                handle_karma(r.body, &mut *scores, |u, s, c| {
                    let msg = format!("{} now at {}", u, s);
                    let channel = format!("#{}", c);
                    let payload = OutgoingWebhook {
                        text: msg.as_slice(),
                        channel: channel.as_slice(),
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
