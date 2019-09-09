#[macro_use]
extern crate warp;

mod activitypub;
mod user_repository;
mod webfinger;

use crate::user_repository::{InMemoryUserRepository, UserRepository};
use futures_util::compat::Future01CompatExt;
use futures_util::try_future::TryFutureExt;
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use std::error::Error;
use std::sync::Arc;
use structopt::StructOpt;
use warp::Filter;

fn parse_acct<'a>(acct: &'a str) -> Option<(&'a str, &'a str)> {
    let user_with_domain = acct.trim_start_matches("acct:");

    if user_with_domain.len() == acct.len() {
        return None;
    }

    let mut parts = user_with_domain.splitn(2, '@');
    if let (Some(user), Some(domain)) = (parts.next(), parts.next()) {
        return Some((user, domain));
    } else {
        return None;
    }
}

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(long, validator = validate_domain)]
    domain: String,
    #[structopt(short = "p", long, default_value = "9090")]
    port: u16,
    #[structopt(long, help = "use http instead of https for external-facing URLs")]
    no_ssl: bool,
}

fn validate_domain(domain: String) -> Result<(), String> {
    if domain.contains("://") {
        Err("domain should not contain a protocol".to_string())
    } else {
        Ok(())
    }
}

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);

    let args = Args::from_args();
    let domain = args.domain;
    let repo = Arc::new(InMemoryUserRepository::new(domain.clone()));

    let https = HttpsConnector::new().unwrap();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let server = warp::serve(routes(domain, repo, !args.no_ssl, client))
        .try_bind(([127, 0, 0, 1], args.port))
        .compat();

    log::info!("Starting server on port {}", args.port);
    if server.await == Err(()) {
        panic!("server error");
    }

    Ok(())
}

#[derive(Clone, Deserialize, Debug)]
struct WebFingerParams {
    resource: String,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CreateNote {
    inbox: String,               // E.g., "https://mastodon.social/inbox"
    in_reply_to: Option<String>, // E.g., "https://mastodon.social/@walfie/9941166"
    content: String,
}

type HttpsClient = hyper::Client<HttpsConnector<hyper::client::HttpConnector>, hyper::Body>;
type ArcRepo = Arc<InMemoryUserRepository>;
fn routes(
    domain: String,
    repo: ArcRepo,
    https: bool,
    client: HttpsClient,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let user_repo = warp::any().map(move || repo.clone());

    let domain_with_protocol = format!("{}://{}", if https { "https" } else { "http" }, &domain);
    let domain_with_protocol = warp::any().map(move || domain_with_protocol.clone());

    let domain = warp::any().map(move || domain.clone());

    let client = warp::any().map(move || client.clone());

    let well_known = path!(".well-known" / "webfinger")
        .and(user_repo.clone())
        .and(warp::query::<WebFingerParams>())
        .and_then(|repo: ArcRepo, params: WebFingerParams| {
            (|| {
                if let Some((name, domain)) = parse_acct(&params.resource) {
                    let user = repo.get_user(&name)?;
                    let wf_doc = webfinger::WebFinger::from_user(&user, &domain);
                    let wf_json = serde_json::to_string(&wf_doc)?;
                    Ok(http::Response::builder()
                        .header("content-type", "application/json")
                        .body(wf_json)
                        .unwrap())
                } else {
                    Ok(http::Response::builder()
                        .status(http::StatusCode::NOT_FOUND)
                        .body("".to_string())
                        .unwrap())
                }
            })()
            .map_err(|e: Box<dyn Error + Send + Sync>| warp::reject::custom(e))
        });

    // TODO: Check `accept` header is `application/activity+json`
    let actor = path!("users" / String)
        .and(domain)
        .and(user_repo.clone())
        .and_then(|username: String, domain: String, repo: ArcRepo| {
            (|| {
                let user = repo.get_user(&username)?;
                let person = activitypub::Person::from_user(&user, &domain)?;
                let person_json = serde_json::to_string(&person)?;

                Ok(http::Response::builder()
                    .header("content-type", "application/activity+json")
                    .body(person_json)
                    .unwrap())
            })()
            .map_err(|e: Box<dyn Error + Send + Sync>| warp::reject::custom(e))
        });

    let create_note = path!("users" / String / "notes" / String)
        .and(warp::body::json())
        .and(domain_with_protocol)
        .and(client)
        .and(user_repo.clone())
        .and_then(
            |username: String,
             id: String,
             note: CreateNote,
             domain_with_protocol: String,
             client: HttpsClient,
             repo: ArcRepo| {
                let request = (|| {
                    use openssl::hash::MessageDigest;
                    use openssl::pkey::PKey;

                    let actor = format!("{}/users/{}", &domain_with_protocol, &username);
                    let id_ref = format!("{}/notes/{}", &actor, &id);
                    let key_id = format!("{}#main-key", &actor);

                    let user = repo.get_user(&username)?;
                    let private_key = PKey::private_key_from_der(&user.private_key)?;

                    let create = activitypub::Create {
                        id: id_ref.clone(),
                        context: "https://www.w3.org/ns/activitystreams".into(),
                        r#type: "Create".into(),
                        actor: actor.clone(),
                        object: activitypub::Note {
                            id: id_ref,
                            r#type: "Note".into(),
                            attributed_to: actor.clone(),
                            to: "https://www.w3.org/ns/activitystreams#Public".into(),
                            content: note.content,
                            in_reply_to: note.in_reply_to,
                        },
                    };

                    let create_json = serde_json::to_string(&create)?;

                    let now: String = chrono::Utc::now().format("%a, %d %b %Y %T GMT").to_string();

                    let mut request = http::Request::builder()
                        .method("POST")
                        .uri(&note.inbox)
                        .header("date", now)
                        .body(create_json.into())?;

                    httpsig::add_signature_header(
                        &mut request,
                        &key_id,
                        MessageDigest::sha256(),
                        &private_key,
                    )?;

                    Ok(request)
                })()
                .map_err(|e: Box<dyn Error + Send + Sync>| warp::reject::custom(e));

                log::info!("{:?}", request); // TODO
                use futures_util::future::Either;
                //use futures_util::stream::StreamExt; // TODO: for `concat`
                use warp::reject::custom;

                let result_future = match request {
                    Ok(req) => Either::Left(client.request(req).map_err(|e| custom(e))),
                    Err(e) => {
                        log::error!("{:?}", e);
                        Either::Right(futures_util::future::err(e))
                    }
                };

                result_future
                    .map_ok(|resp| {
                        let (parts, _body) = resp.into_parts();
                        http::Response::from_parts(parts, "".to_string()) // TODO
                    })
                    .compat()
            },
        );

    let get = warp::get2().and(well_known.or(actor));
    let post = warp::post2().and(create_note);
    get.or(post).boxed()
}
