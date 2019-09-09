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

    let server = warp::serve(routes(domain, repo))
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

fn routes(
    domain: String,
    repo: Arc<InMemoryUserRepository>,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let user_repo = warp::any().map(move || repo.clone());
    let domain = warp::any().map(move || domain.clone());

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
                            .status(http::StatusCode::NOT_FOUND)
                            .body("".to_string())
                            .unwrap())
                    }
                })()
                .map_err(|e: Box<dyn Error + Send + Sync>| warp::reject::custom(e))
            },
        );

    // TODO: Check `accept` header is `application/activity+json`
    let actor = path!("users" / String).and(domain).and(user_repo).and_then(
        |username: String, domain: String, repo: Arc<InMemoryUserRepository>| {
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
        },
    );

    warp::get2().and(well_known.or(actor)).boxed()
}
