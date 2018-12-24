extern crate robohome_shared;
extern crate serde_json;
extern crate warp;
extern crate env_logger;
extern crate uuid;

use robohome_shared::{
    data::{
        Flip,
        get_all_switches,
        Switch,
        check_auth,
        check_token,
    },
    Error,
    ipc::send,
};
use serde_json::to_string;
use uuid::Uuid;

use warp::{
    Filter,
    get2,
    body::json,
    http::Response,
    index,
    path,
    put2,
    post2,
    Reply,
    header,
    query,
};

fn main() {

    ::std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let flipping = put2()
        .and(path("flip"))
        .and(json())
        .map(flip_switch);
    let all_switches = get2()
        .and(path("switches"))
        .map(get_switches);
    let switch_flips = post2()
        .and(path("switch"))
        .and(json())
        .map(get_switch_flips);
    let auth = get2()
        .and(path("auth"))
        .and(query())
        .map(check_auth_token);

    let routes = flipping
        .or(switch_flips)
        .or(all_switches)
        .or(auth)
        .or(warp::filters::fs::dir("public"));
    warp::serve(routes.with(warp::log("robohome_flipper")))
        .run(([0,0,0,0], 3434));
}

// fn auth(header: String) -> Result<(), Error> {
//     let split = header.split(' ');
//     let _ = split.next();
//     Ok(())
// }

fn flip_switch(flip: Flip) -> impl Reply {
    match send("switches", &flip) {
        Ok(_) => Response::builder().body(format!(r#"{{flipped: {}}}"#, flip.code)),
        Err(e) => {
            let (status, body) = error_response(&e);
            Response::builder()
                            .status(status)
                            .body(body)
        },
    }
}

fn get_switch_flips(switch: Switch) -> impl Reply {
    let (status, body) = get_switch_flips_response(switch);
    Response::builder()
            .status(status)
            .body(body)
}

fn get_switches() -> impl Reply {
    let (status, body) = get_switches_response();
    Response::builder()
        .status(status)
        .body(body)
}

fn get_switches_response() -> (u16, String) {
    match get_all_switches() {
        Ok(switches) => match to_string(&switches) {
            Ok(body) => (200, body),
            Err(e) => error_response(&Error::from(e))
        },
        Err(e) => error_response(&e)
    }
}

fn get_switch_flips_response(switch: Switch) -> (u16, String) {
    match robohome_shared::data::get_flips_for_switch(switch.id) {
        Ok(flips) => match to_string(&flips) {
            Ok(body) => (200, body),
            Err(e) => error_response(&Error::from(e))
        },
        Err(e) => error_response(&e),
    }
}
use std::collections::HashMap;
fn check_auth_token(query: HashMap<String, String>) -> impl Reply {
    let mut builder = Response::builder();
    let token = if let Some(token) = query.get("token") {
        if let Ok(token) = Uuid::parse_str(token) {
            token
        } else {
            builder.status(401);
            return builder.body(String::new());
        }
    } else {
        builder.status(401);
        return builder.body(String::new());
    };
    let body = match check_auth(&token) {
        Ok(valid) => {
            if valid {
                builder.status(200);
                format!(r#"<html><head></head><body><script>(function() {{
                    localStorage.setItem('auth-token', '{}');
                    location = '/authorize/submit.html';
                }})()</script></body></html>"#, token.to_string())
            } else {
                builder.status(401);
                return builder.body(String::new());
            }
        },
        Err(e) => {
            let (status, body) = error_response(&e);
            builder.status(status);
            body
        },
    };
    builder.body(body)
}

fn error_response(e: &Error) -> (u16, String) {
    (500, to_string(e).unwrap_or(format!(r#"{{ messsage: "{}" }}"#, e)))
}