use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use crate::ir::Value;

pub fn http_server_create(args: &[Value]) -> Result<Value, String> {
    let port = args.first().and_then(|v| match v {
        Value::Int(n) => Some(*n as u16),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }).unwrap_or(8080);
    Ok(Value::String(format!("server_{port}")))
}

pub fn http_server_route(args: &[Value]) -> Result<Value, String> {
    let _server = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let method = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("GET");
    let path = args.get(2).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("/");
    Ok(Value::Map(vec![
        (Value::String("method".into()), Value::String(method.into())),
        (Value::String("path".into()), Value::String(path.into())),
        (Value::String("status".into()), Value::String("registered".into())),
    ]))
}

pub fn http_server_static(args: &[Value]) -> Result<Value, String> {
    let _server = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _path = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("/");
    let _dir = args.get(2).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("./public");
    Ok(Value::Bool(true))
}

pub fn http_server_listen(args: &[Value]) -> Result<Value, String> {
    let addr = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("127.0.0.1:8080");
    Ok(Value::String(format!("listening on {addr}")))
}

pub fn http_response(args: &[Value]) -> Result<Value, String> {
    let status = args.first().and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(200);
    let body = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status)),
        (Value::String("body".into()), Value::String(body.into())),
        (Value::String("content_type".into()), Value::String("text/plain".into())),
    ]))
}

pub fn http_response_json(args: &[Value]) -> Result<Value, String> {
    let status = args.first().and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(200);
    let body = args.get(1).cloned().unwrap_or(Value::Null);
    let json_str = format!("{body:?}");
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status)),
        (Value::String("body".into()), Value::String(json_str)),
        (Value::String("content_type".into()), Value::String("application/json".into())),
    ]))
}

pub fn http_response_html(args: &[Value]) -> Result<Value, String> {
    let status = args.first().and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(200);
    let body = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status)),
        (Value::String("body".into()), Value::String(body.into())),
        (Value::String("content_type".into()), Value::String("text/html; charset=utf-8".into())),
    ]))
}

pub fn http_response_error(args: &[Value]) -> Result<Value, String> {
    let status = args.first().and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(500);
    let message = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("Internal Server Error");
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status)),
        (Value::String("body".into()), Value::String(message.into())),
        (Value::String("content_type".into()), Value::String("text/plain".into())),
    ]))
}

pub fn http_request_method(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::String("GET".into()))
}

pub fn http_request_path(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("/");
    Ok(Value::String("/".into()))
}

pub fn http_request_query(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Map(vec![]))
}

pub fn http_request_body(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::String("".into()))
}

pub fn http_request_header(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _name = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Null)
}
