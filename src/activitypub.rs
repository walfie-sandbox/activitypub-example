use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Person<'a> {
    #[serde(rename = "@context", default)]
    pub context: Vec<&'a str>,
    pub id: &'a str,
    pub r#type: &'a str,
    pub preferred_username: &'a str,
    pub inbox: &'a str,
    pub public_key: PublicKey<'a>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey<'a> {
    pub id: &'a str,
    pub owner: &'a str,
    pub public_key_pem: &'a str,
}
