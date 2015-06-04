use hyper;
use url::{Url,form_urlencoded};
use std::str;
use std::io::{Read,Write};
use rustc_serialize::json;
use hyper::{header,client,method};
use rustc_serialize::Encodable;


#[derive(RustcEncodable)]
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
    pub fn send<'a>(&self, payload: &OutgoingWebhook) {
        let url = Url::parse(&self.endpoint_url()[..]).ok().expect("Invalid URL :-(");

        let mut request = client::Request::new(method::Method::Post, url).unwrap();

        let json: String = json::encode(&payload).ok().unwrap();
        let form_payload = [ ("payload".to_string(), json) ];
        let body: String = form_urlencoded::serialize(&form_payload);

        {
            let mut headers = request.headers_mut();
            headers.set(header::ContentType::form_url_encoded());
        }

        let mut request = request.start().unwrap();

        request.write_all(body.as_bytes());

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
