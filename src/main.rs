use crate::sysinfo::SysInfo;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};

use crate::graphql::{Query, RootSchema};
use anyhow::{Context, Result};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use std::convert::Infallible;
use std::net::SocketAddr;

mod graphql;
mod sysinfo;

async fn query(req: Request<Body>, schema: RootSchema) -> Result<Response<Body>> {
    let body = hyper::body::to_bytes(req.into_body()).await?;
    let body = std::str::from_utf8(body.as_ref())?;
    let body = serde_json::from_str::<serde_json::Value>(body)?;

    let q = body
        .get("query")
        .context("no query field")?
        .as_str()
        .context("wrong field type")?;

    let response = schema.execute(q).await;

    let response = response.data.into_json()?;
    Ok(Response::new(serde_json::to_string(&response)?.into()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let sys_info = SysInfo::new();
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(sys_info)
        .finish();

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;

    let mk_srv = make_service_fn(move |_| {
        let schema = schema.clone();
        async {
            Ok::<_, Infallible>(service_fn(move |req| {
                let schema = schema.clone();
                async move { query(req, schema.clone()).await }
            }))
        }
    });

    let _ = hyper::Server::bind(&addr).serve(mk_srv).await;

    Ok(())
}
