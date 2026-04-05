use axum::{
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use ipnet::IpNet;
use std::net::{IpAddr, SocketAddr};

use crate::settings::SharedConfig;

pub async fn ip_filter_middleware(
    State(config): State<SharedConfig>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let cidr: IpNet = {
        let state = config.lock().unwrap_or_else(|e| e.into_inner());
        state
            .config
            .api_config
            .accepted_ip_range
            .parse()
            .unwrap_or_else(|_| "0.0.0.0/0".parse().unwrap())
    };

    let client_ip = match addr.ip() {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(IpAddr::V6(v6)),
        ip => ip,
    };

    if cidr.contains(&client_ip) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

pub async fn auth_middleware(
    State(config): State<SharedConfig>,
    headers: axum::http::HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    let valid = {
        let state = config.lock().unwrap_or_else(|e| e.into_inner());
        state
            .config
            .api_config
            .api_keys
            .iter()
            .any(|k| k.key == token)
    };

    if valid {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
