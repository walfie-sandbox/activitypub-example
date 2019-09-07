use crate::user_repository::User;
use ifmt::iformat;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub rel: String,
    pub r#type: Cow<'static, str>,
    pub href: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebFinger {
    pub subject: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub links: Vec<Link>,
}

impl WebFinger {
    pub fn from_user<'a>(user: &'a User, domain: &'a str) -> Self {
        let uri = iformat!("{domain}/users/{user.username}");

        WebFinger {
            subject: iformat!("acct:{user.username}@{domain}"),
            aliases: vec![uri.clone()],
            links: vec![Link {
                rel: "self".into(),
                r#type: "application/activity+json".into(),
                href: uri,
            }],
        }
    }
}
