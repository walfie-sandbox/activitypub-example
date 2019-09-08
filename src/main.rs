#[macro_use]
extern crate warp;

mod activitypub;
mod user_repository;
mod webfinger;

use crate::user_repository::UserRepository;

use futures_util::compat::Future01CompatExt;
use serde::Deserialize;
use std::error::Error;
use warp::{Filter, Future};

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();
    /*
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
    */

    let server = {
        let well_known = path!(".well-known" / "webfinger")
            .and(warp::query::<WebFingerParams>())
            .map(|params| "test");

        let actor = path!("users" / String).map(|username| "test");
        let any = warp::any().map(|| "???");

        let routes = warp::get2().and(well_known.or(actor).or(any));
        warp::serve(routes)
    }
    .try_bind(([127, 0, 0, 1], 9090))
    .boxed()
    .compat();

    server.await.map_err(|e| {
        eprintln!("error");
    });

    Ok(())
}

#[derive(Deserialize, Debug)]
struct WebFingerParams {
    resource: String,
}

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
