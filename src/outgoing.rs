use hyper;

use url::{Url,form_urlencoded};

use std::str;

use std::io::Read;

use serialize::json;
use serialize::serialize::Encodable;

use hyper::header;


#[derive(Encodable)]
pub struct OutgoingWebhook<'a> {
    pub text: &'a str,
    pub channel: &'a str,
    pub username: &'a str,
    pub icon_emoji: Option<&'a str>,
}

pub struct SlackEndpoint {
    pub url: String,
}

impl SlackEndpoint {
    fn endpoint_url(&self) -> &String {
        &self.url // Lurk because there seem to be several formats
    }

    #[allow(unused_must_use)]
    pub fn send(&self, payload: &OutgoingWebhook) {
        let mut client = hyper::Client::new();
        let url = Url::parse(&self.endpoint_url()[..]).ok().expect("Invalid URL :-(");

        let mut request = client.post(url);

        let json = json::encode(payload).ok().unwrap();
        let payload = [ ("payload".to_string(), json) ];
        let body = form_urlencoded::serialize(&payload);
        let length = body.as_bytes().len();

        request.body(body.as_bytes());
        request.header(header::ContentType::form_url_encoded());

        match request.send() {
            Err(_) => println!("Error => couldn't read response"),
            Ok(mut resp) => {
                let mut buf: &mut [u8] = &mut [0; 1024];
                resp.read(buf);
                let buf = str::from_utf8(buf).unwrap();
                println!("Got status: {} => {}", resp.status, buf);
            }
        }

    }
}
