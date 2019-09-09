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
    pub context: Vec<Cow<'static, str>>, // TODO: context can also contain JSON objects
    pub id: String,
    pub r#type: Cow<'static, str>,
    pub preferred_username: String,
    pub inbox: String,
    pub public_key: PublicKey,
}

impl Person {
    pub fn from_user<'a>(
        user: &'a User,
        domain: &'a str,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub r#type: Cow<'static, str>,
    pub attributed_to: String,
    pub to: String,
    pub content: String,
    pub in_reply_to: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Create {
    #[serde(rename = "@context", default)]
    pub context: Cow<'static, str>,
    pub r#type: Cow<'static, str>,
    pub id: String,
    //pub to: Vec<String>,
    pub actor: String,
    pub object: Note, // TODO: Other types
}
