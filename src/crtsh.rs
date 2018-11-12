use super::common::sha256_hex;
use super::ct::AddChainRequest;
use base64;
use futures::compat::Future01CompatExt;
use futures::prelude::*;
use hyper;
use hyper::rt::Stream;
use serde_json;
use url;

pub fn build_chain_for_cert<C: hyper::client::connect::Connect + 'static>(
    http_client: &hyper::Client<C>,
    cert: &[u8],
) -> impl Future<Output = Result<Vec<Vec<u8>>, ()>> {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("b64cert", &base64::encode(&cert))
        .append_pair("onlyonechain", "Y")
        .finish();
    let request = hyper::Request::builder()
        .method("POST")
        .uri("https://crt.sh/gen-add-chain")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Connection", "keep-alive")
        .body(body.into_bytes().into())
        .unwrap();
    // TODO: undo this once lifetime bugs are fixed
    let r = http_client.request(request);
    async {
        let response = match await!(r.compat()) {
            Ok(response) => response,
            // TODO: maybe be more selective in error handling
            Err(_) => return Err(()),
        };

        if response.status() == hyper::StatusCode::NOT_FOUND {
            return Err(());
        }

        let body = await!(response.into_body().concat2().compat()).unwrap();
        let add_chain_request: AddChainRequest = serde_json::from_slice(&body).unwrap();
        Ok(
            add_chain_request
                .chain
                .iter()
                .map(|c| base64::decode(c).unwrap())
                .collect(),
        )
    }
}

pub fn is_cert_logged<C: hyper::client::connect::Connect + 'static>(
    http_client: &hyper::Client<C>,
    cert: &[u8],
) -> impl Future<Output = Result<bool, ()>> {
    let request = hyper::Request::builder()
        .method("GET")
        .uri(format!("https://crt.sh/?d={}", sha256_hex(cert)))
        .header("Connection", "keep-alive")
        .body(hyper::Body::empty())
        .unwrap();
    let r = http_client.request(request);
    async {
        let response = await!(r.compat()).unwrap();
        Ok(response.status() == hyper::StatusCode::OK)
    }
}

pub fn url_for_cert(cert: &[u8]) -> String {
    format!("https://crt.sh?q={}", sha256_hex(cert).to_uppercase())
}
