use serde::{Deserialize, Serialize};
use trillium::{Conn, Handler, Headers, KnownHeaderName};
use trillium_api::*;
use trillium_testing::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
struct Struct {
    string: String,
    numbers: Option<Vec<usize>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse {
    s: Struct,
}

fn app_with_body() -> impl Handler {
    api(|__: &mut Conn, Body(mut s): Body<Struct>| async move {
        if let Some(numbers) = &mut s.numbers {
            numbers.push(100);
        }
        Body(ApiResponse { s })
    })
}

#[test]
fn json_request_json_response() {
    assert_ok!(
        get("/")
            .with_request_header("content-type", "application/json")
            .with_request_body(r#"{"string": "string", "numbers": [ 1, 2, 3]}"#)
            .on(&app_with_body()),
        r#"{"s":{"string":"string","numbers":[1,2,3,100]}}"#
    );
}

#[test]
fn form_urlencoded_json_response() {
    assert_ok!(
        get("/")
            .with_request_header("content-type", "application/x-www-form-urlencoded")
            .with_request_body(r#"string=string"#)
            .on(&app_with_body()),
        r#"{"s":{"string":"string","numbers":null}}"#
    );
}

#[test]
fn malformed_json_request() {
    assert_response!(
        get("/")
            .with_request_header("content-type", "application/json")
            .with_request_body(r#"this is not valid json"#)
            .on(&app_with_body()),
        422,
        r#"{"error":{"message":"expected ident at line 1 column 2","path":".","type":"parse_error"}}"#
    );
}

fn app_without_body() -> impl Handler {
    api(|_: &mut Conn, _: ()| async { Json(json!({"health": "ok" })) })
}

#[test]
fn get_json_response() {
    assert_ok!(
        get("/").on(&app_without_body()),
        r#"{"health":"ok"}"#,
        "Content-Type" => "application/json"
    );
}

#[test]
fn get_custom_content_type() {
    assert_ok!(
        get("/").on(&(
            Headers::from_iter([(KnownHeaderName::ContentType, "application/custom+json")]),
            Json(json!({"health": "ok"}))
        )),
        r#"{"health":"ok"}"#,
        "Content-Type" => "application/custom+json"
    );
}
