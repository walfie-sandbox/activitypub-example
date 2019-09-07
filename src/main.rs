mod activitypub;
mod user_repository;
mod webfinger;

use crate::user_repository::UserRepository;
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

    let mut repo = user_repository::InMemoryUserRepository::new(domain.to_owned());
    let user = repo.get_user(&username).await?;

    let person = activitypub::Person::from_user(&user, &domain)?;
    let person_json = serde_json::to_string(&person)?;
    dbg!(&person_json);

    let wf_doc = webfinger::WebFinger::from_user(&user, &domain);
    let wf_json = serde_json::to_string(&wf_doc)?;
    dbg!(&wf_json);

    Ok(())

    /*
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
        uri = uri
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
        */
}
