use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebFinger<'a> {
    pub subject: &'a str,
    #[serde(default)]
    pub aliases: Vec<&'a str>,
    #[serde(default)]
    pub links: Vec<Link<'a>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Link<'a> {
    pub rel: &'a str,
    pub r#type: &'a str,
    pub href: &'a str,
}
