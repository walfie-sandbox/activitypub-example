#[macro_use]
extern crate warp;

mod activitypub;
mod user_repository;
mod webfinger;

use crate::user_repository::{InMemoryUserRepository, UserRepository};
use futures_util::compat::Future01CompatExt;
use serde::Deserialize;
use std::error::Error;
use std::sync::Arc;
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

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();

    let domain = "https://example.com";

    let repo = Arc::new(InMemoryUserRepository::new(domain.to_owned()));

    let server = {
        let user_repo = warp::any().map(move || repo.clone());
        let domain = warp::any().map(move || domain);

        let well_known = path!(".well-known" / "webfinger")
            .and(user_repo.clone())
            .and(warp::query::<WebFingerParams>())
            .and_then(
                |repo: Arc<InMemoryUserRepository>, params: WebFingerParams| {
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
                                .status(http::StatusCode::BAD_REQUEST)
                                .body("".to_string())
                                .unwrap())
                        }
                    })()
                    .map_err(|e: Box<dyn Error + Send + Sync>| warp::reject::custom(e))
                },
            );

        // TODO: Check `accept` header is `application/activity+json`
        let actor = path!("users" / String).and(domain).and(user_repo).and_then(
            |username: String, domain: &str, repo: Arc<InMemoryUserRepository>| {
                (|| {
                    let user = repo.get_user(&username)?;
                    let person = activitypub::Person::from_user(&user, domain)?;
                    let person_json = serde_json::to_string(&person)?;

                    Ok(http::Response::builder()
                        .header("content-type", "application/activity+json")
                        .body(person_json)
                        .unwrap())
                })()
                .map_err(|e: Box<dyn Error + Send + Sync>| warp::reject::custom(e))
            },
        );

        // TODO: Recover
        let routes = warp::get2().and(well_known.or(actor)).boxed();
        warp::serve(routes)
    }
    .try_bind(([127, 0, 0, 1], 9090))
    .compat();

    if server.await == Err(()) {
        panic!("server error");
    }
    Ok(())
}

#[derive(Clone, Deserialize, Debug)]
struct WebFingerParams {
    resource: String,
}
