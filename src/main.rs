#![feature(slicing_syntax)]
#![feature(overloaded_calls)]

extern crate hyper;
extern crate url;
extern crate rustc_serialize;
extern crate serialize;

use std::os;
use std::env;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::net::{SocketAddr, Ipv4Addr};
use std::collections::HashMap;

use serialize::json;

use hyper::{Get, Post};
use hyper::header::ContentLength;
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;

use incoming::{SlackPayload};
use outgoing::{SlackEndpoint, OutgoingWebhook};

mod incoming;
mod outgoing;

type Scores = HashMap<String, i32>;

#[derive(Clone)]
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

fn handle_karma<F>(req: Vec<u8>, points: &mut Scores, cb: F)
    where F: Fn(&&str, &i32, &&str) {
    let payload = match SlackPayload::from_body(req.as_slice()) {
        Ok(payload) => payload,
        Err(err) => {
            println!("Error! {}", err); return
        }
    };

    let op: Box<Fn(i32) -> i32> = match payload.command {
        "%2F%2B%2B" => Box::new(|c: i32| c + 1),
        "%2F++" => Box::new(|c: i32| c + 1),
        "%2F%2D%2D" => Box::new(|c: i32| c - 1),
        "%2F--" => Box::new(|c: i32| c - 1),
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

fn echo(mut req: Request, mut res: Response) {
    match req.uri {
        AbsolutePath(ref path) => match (&req.method, &path[..]) {
            (&Get, "/") => {
                let res = res.start().unwrap();
                res.write(b"Hello world!");
                res.end();
            },
            // (&Get, "/karma") => {
            //     let scores = (*self.scores).lock();
            //     let json = json::encode(&*scores);
            //     w.write(json.as_bytes());
            // }
            // (&Post, "/normalise") => {
            //     let mut scores = (*self.scores).lock();
            //     for (_, v) in scores.iter_mut() {
            //         let new = *v as f64;
            //         *v = new.sqrt() as i32;
            //     }
            // }
            // (&Post, "/slack") => {
            //     let mut scores = (*self.scores).lock();
            //     let slack = self.get_slack_endpoint();
            //     handle_karma(r.body, &mut *scores, |u, s, c| {
            //         let msg = format!("{} now at {}", u, s);
            //         let channel = format!("#{}", c);
            //         let payload = OutgoingWebhook {
            //             text: msg.as_slice(),
            //             channel: channel.as_slice(),
            //             username: "karmabot",
            //             icon_emoji: Some(":ghost:"),
            //         };

            //         slack.send(&payload);
            //     });
            // },
            (_, _) => {
                *res.status_mut() = hyper::NotFound;
                let res = res.start().unwrap();
                res.write(b"Not found :(");
                res.end();
            }
        }
    }
}

impl KarmaServer {
    fn get_slack_endpoint(&self) -> SlackEndpoint {
        SlackEndpoint {
            url: self.endpoint.clone()
        }
    }
}

fn main() {
    let endpoint = match env::var_os("WEBHOOK_ENDPOINT") {
        Some(e) => e,
        None => panic!("Must set WEBHOOK_ENDPOINT"),
    };
    let server = KarmaServer::new(HashMap::new(), endpoint.into_string().unwrap());
    server.serve_forever();
}
