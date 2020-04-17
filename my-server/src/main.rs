#![deny(warnings)]

use handlebars::Handlebars;
use headers::HeaderMapExt;
use my_types::{IncrementReq, IncrementResp};
use serde::Serialize;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::RwLock;
use warp::Filter;
use serde_json::json;

/// A serialized message to report in JSON format.
#[derive(Serialize)]
struct ErrorMessage<'a> {
    code: u16,
    message: &'a str,
}

// simple counter - could be an atomic, but I wanted something that would motivate use of async in resp handlers
static mut GLOBAL_STATE: Option<Arc<RwLock<u32>>> = None;

fn get_state() -> Arc<RwLock<u32>> {
    unsafe {
        match &GLOBAL_STATE {
            Some(x) => x.clone(),
            None => panic!("global ctx not set"),
        }
    }
}

#[tokio::main]
async fn main() {
    let lock = RwLock::new(0);
    unsafe {
        GLOBAL_STATE = Some(Arc::new(lock));
    }

    let index_route = warp::get().and(warp::path::end()).and_then(|| async move {
        let state = get_state();
        let counter = state.read().await;
        let resp: Result<_, warp::Rejection> =
            Ok(render_index(*counter));
        resp
    });

    let post_route = warp::post()
        .and(warp::path("increment"))
        .and(warp::path("counter"))
        .and(warp::body::content_length_limit(1024 * 16)) // arbitrary? what if I just drop this?
        .and(warp::body::json())
        // .end() why is this commented out?
        .and_then(|req: IncrementReq| async move {
            let state = get_state();
            let mut counter = state.write().await;
            *counter += req.increment_counter_by;

            let resp = IncrementResp {
                new_counter_state: *counter,
            };

            let resp: Result<_, warp::Rejection> = Ok(warp::reply::json(&resp));
            resp
        });

    // serve static content embeded in binary
    let static_route = warp::get()
        .and(warp::path::param::<String>()) // FIXME: does this work with nested paths?
        .map(
            |path: String| match my_frontend::get_static_asset(&path) {
                None => {
                    hyper::Response::builder()
                        .status(hyper::StatusCode::NOT_FOUND)
                        .body(hyper::Body::empty())
                        .unwrap()
                }
                Some(blob) => {
                    let body = hyper::Body::from(blob);
                    let mut resp = hyper::Response::new(body);

                    let mime_type = mime_guess::from_path(path).first_or_octet_stream();
                    resp.headers_mut()
                        .typed_insert(headers::ContentType::from(mime_type));
                    resp.headers_mut()
                        .typed_insert(headers::AcceptRanges::bytes());
                    resp.headers_mut().typed_insert(headers::ContentLength(blob.len() as u64));

                    resp
                }
            },
        );

    let routes = post_route.or(index_route).or(static_route);

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    println!("running on localhost:8080");
    warp::serve(routes).run(socket).await;
}

pub fn render_index(counter_value: u32) -> impl warp::Reply {
    let hb: Handlebars = {
        let template = r#"<!doctype html>
            <html>
                <head>
                    <meta charset="utf-8" />
                    <title>My App</title>
                    <link rel="stylesheet" href="/tree.css" >
                </head>
                <body>
                    <script>
                        window.initial_counter_state="{{initial_counter_state}}";
                    </script>
                    <script src="/app.js"></script>
                </body>
            </html>"#;

        let mut hb = Handlebars::new();
        hb.register_template_string("index.html", template).unwrap();
        hb
    };

    let body = hb
        .render("index.html", &json!({ "initial_counter_state": format!("{}", counter_value) }))
        .unwrap_or_else(|err| err.to_string());

    warp::reply::html(body)
}
