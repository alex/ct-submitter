use super::common::sha256_hex;
use super::ct::AddChainRequest;
use base64;
use futures::prelude::*;
use hyper;
use serde_json;
use url;

#[async]
pub fn build_chain_for_cert<C: hyper::client::Connect>(
    http_client: &hyper::Client<C>,
    cert: &[u8],
) -> Option<Vec<Vec<u8>>> {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("b64cert", &base64::encode(cert))
        .append_pair("onlyonechain", "Y")
        .finish();
    let body_bytes = body.as_bytes();
    let response = match http_client
        .post("https://crt.sh/gen-add-chain")
        .header(hyper::header::ContentType::form_url_encoded())
        .header(hyper::header::Connection::keep_alive())
        .body(hyper::client::Body::BufBody(body_bytes, body_bytes.len()))
        .send() {
        Ok(response) => response,
        // TODO: maybe be more selective in error handling
        Err(_) => return None,
    };

    if response.status == hyper::StatusCode::NotFound {
        return None;
    }

    let add_chain_request: AddChainRequest = serde_json::from_reader(response).unwrap();
    Some(
        add_chain_request
            .chain
            .iter()
            .map(|c| base64::decode(c).unwrap())
            .collect(),
    )
}

#[async]
pub fn is_cert_logged<C: hyper::client::Connect>(
    http_client: &hyper::Client<C>,
    cert: &[u8],
) -> bool {
    let response = http_client
        .get(&format!("https://crt.sh/?d={}", sha256_hex(cert)))
        .header(hyper::header::Connection::keep_alive())
        .send()
        .unwrap();
    response.status == hyper::StatusCode::Ok
}

pub fn url_for_cert(cert: &[u8]) -> String {
    format!("https://crt.sh?q={}", sha256_hex(cert).to_uppercase())
}
