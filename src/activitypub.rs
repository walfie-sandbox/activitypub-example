use crate::user_repository::User;
use ifmt::iformat;
use openssl::rsa::Rsa;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(rename = "@context", default)]
    pub context: Vec<Cow<'static, str>>,
    pub id: String,
    pub r#type: Cow<'static, str>,
    pub preferred_username: String,
    pub inbox: String,
    pub public_key: PublicKey,
}

impl Person {
    pub fn from_user<'a>(user: &'a User, domain: &'a str) -> Result<Self, Box<dyn Error>> {
        let uri = iformat!("{domain}/users/{user.username}");

        let pubkey_pem =
            String::from_utf8(Rsa::public_key_from_der(&user.public_key)?.public_key_to_pem()?)?;

        Ok(Person {
            id: uri.clone(),
            context: vec![
                "https://www.w3.org/ns/activitystreams".into(),
                "https://w3id.org/security/v1".into(),
            ],
            r#type: "Person".into(),
            preferred_username: user.username.clone(),
            inbox: iformat!("{uri}/inbox"),
            public_key: PublicKey {
                id: iformat!("{uri}#main-key"),
                owner: uri,
                public_key_pem: pubkey_pem,
            },
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    pub id: String,
    pub owner: String,
    pub public_key_pem: String,
}
