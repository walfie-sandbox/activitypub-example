mod activitypub;
mod webfinger;

use futures_util::TryStreamExt;
use http_signatures::{CreateKey, ShaSize};
use hyper::header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Body, Client, Request};
use ifmt::iformat;
use openssl::rsa::Rsa;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let domain = "https://example.com";
    let username = "example_user";

    let uri = iformat!("{domain}/users/{username}");

    let keypair = Rsa::generate(4096)?;

    let pubkey = keypair.public_key_to_der()?;
    let privkey = keypair.private_key_to_der()?;

    let pubkey_pem = String::from_utf8(Rsa::public_key_from_der(&pubkey)?.public_key_to_pem()?)?;

    let person = activitypub::Person {
        context: vec![
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1",
        ],
        id: &uri,
        r#type: "Person",
        preferred_username: &username,
        inbox: &iformat!("{uri}/inbox"),
        public_key: activitypub::PublicKey {
            id: &iformat!("{uri}#main-key"),
            owner: &uri,
            public_key_pem: &pubkey_pem,
        },
    };

    let person_json = serde_json::to_string(&person)?;

    let wf_doc = webfinger::WebFinger {
        subject: &iformat!("acct:{username}@{domain}"),
        aliases: vec![&uri],
        links: vec![webfinger::Link {
            rel: "self",
            r#type: "application/activity+json",
            href: &uri,
        }],
    };

    let webfinger_json = serde_json::to_string(&wf_doc)?;

    use ring::signature::RSAKeyPair;
    let json = format!(
        r#"
        {{
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": "{uri}/statuses/123",
            "type": "Create",
            "actor": "{uri}",
            "object": {{
                "id": "{uri}/statuses/123",
                "type": "Note",
                "published": "2018-06-23T17:17:11Z",
                "attributedTo": "{uri}",
                "inReplyTo": "https://example.com/@user/123",
                "content": "<p>Hello world</p>",
                "to": "https://www.w3.org/ns/activitystreams#Public"
            }}
        }}
        "#,
        uri = uri,
        domain = domain
    );
    let mut req = Request::post("http://localhost:3000")
        .body(json.to_owned().into())
        .unwrap();
    req.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_str("application/json").unwrap(),
    );

    use http_signatures::prelude::WithHttpSignature;
    req.headers_mut().insert(
        CONTENT_LENGTH,
        HeaderValue::from_str(&format!("{}", json.len())).unwrap(),
    );

    let key_id = &uri;
    // Add the HTTP Signature
    let private_key = RSAKeyPair::from_der(untrusted::Input::from(&privkey)).unwrap();
    req.with_signature_header(key_id.into(), CreateKey::rsa(private_key, ShaSize::SHA256))
        .unwrap();

    let client = Client::new();
    let res = client.request(req).await?;
    println!("POST: {}", res.status());

    let response_body = res.into_body().try_concat().await?;
    Ok(())
}
