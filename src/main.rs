mod activitypub;
mod webfinger;

use ifmt::iformat;
use openssl::rsa::Rsa;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let domain = "https://example.com";
    let username = "example_user";

    let uri = iformat!("{domain}/users/{username}");

    let keypair = Rsa::generate(4096)?;

    let pubkey = keypair.public_key_to_der()?;
    let privkey = keypair.private_key_to_der()?;

    let pubkey_pem = String::from_utf8(Rsa::public_key_from_der(&pubkey)?.public_key_to_pem()?)?;
    dbg!(&pubkey_pem);

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

    let json = serde_json::to_string(&person)?;
    dbg!(&json);

    let wf_doc = webfinger::WebFinger {
        subject: &iformat!("acct:{username}@{domain}"),
        aliases: vec![&uri],
        links: vec![webfinger::Link {
            rel: "self",
            r#type: "application/activity+json",
            href: &uri,
        }],
    };

    let json = serde_json::to_string(&wf_doc)?;
    dbg!(&json);

    Ok(())
}
