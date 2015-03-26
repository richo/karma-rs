use http::client::RequestWriter;
use http::method::Post;
use std::str;
use url::Url;
use url::form_urlencoded;

use serialize::json;
use serialize::Encodable;


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
    pub fn send(&self, payload: &OutgoingWebhook) {
        let url = Url::parse(self.endpoint_url().as_slice()).ok().expect("Invalid URL :-(");
        let mut request: RequestWriter = RequestWriter::new(Post, url).unwrap();
        let json = json::encode(payload);
        let payload = [ ("payload".to_string(), json) ];
        let body = form_urlencoded::serialize_owned(payload);

        let length = body.as_bytes().len();
        request.headers.insert_raw("Content-Type".to_string(), b"application/x-www-form-urlencoded");
        request.headers.insert_raw("Content-Length".to_string(), length.to_string().as_bytes());

        request.write(body.as_bytes());

        match request.read_response() {
            Err(_) => println!("Error => couldn't read response"),
            Ok(mut resp) => {
                let mut buf: &mut [u8] = &mut [0, ..1024];
                resp.read(buf);
                let buf = str::from_utf8(buf).unwrap();
                println!("Got status: {} => {}", resp.status, buf);
            }
        }

    }
}
