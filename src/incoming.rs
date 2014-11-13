use std::str;
use std::collections::HashMap;

#[deriving(Show)]
pub struct SlackPayload<'a> {
    token: &'a str,
    team_id: &'a str,
    channel_id: &'a str,
    channel_name: &'a str,
    user_id: &'a str,
    pub user_name: &'a str,
    pub command: &'a str,
    pub text: &'a str,
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
    pub fn from_body<'a>(req: &[u8]) -> Result<SlackPayload, String> {
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
