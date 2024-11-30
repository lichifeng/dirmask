use axum::{extract::Path as AxumPath, response::IntoResponse, routing::get, Router};
use rustls_acme::caches::DirCache;
use rustls_acme::AcmeConfig;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let mut state = AcmeConfig::new(["nc.233997.xyz"])
        .contact(["mailto:78833217@qq.com"])
        .cache(DirCache::new("./rustls_acme_cache"))
        .directory_lets_encrypt(false)
        .state();
    let acceptor = state.axum_acceptor(state.default_rustls_config());

    tokio::spawn(async move {
        loop {
            match state.next().await.unwrap() {
                Ok(ok) => log::info!("event: {:?}", ok),
                Err(err) => log::error!("error: {:?}", err),
            }
        }
    });

    let app = Router::new().route("/", get(check_path_none)).route("/*path", get(check_path));

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 443));
    axum_server::bind(addr).acceptor(acceptor).serve(app.into_make_service()).await.unwrap();
}

async fn check_path_none() -> impl IntoResponse {
    check_path(AxumPath(String::new())).await
}

async fn check_path(AxumPath(p): AxumPath<String>) -> impl IntoResponse {
    log::info!("Request path: {:?}", p);
    let formatted_path = format!("./{}", &p);
    let path = Path::new(&formatted_path);

    log::info!("Full path: {:?}", path);
    log::info!("Canonical path: {:?}", path.canonicalize());

    match path.try_exists() {
        Ok(exists) => {
            if exists {
                let file_type = if path.is_dir() { "directory" } else { "file" };
                log::info!("Requested path: {:?}", path.canonicalize());
                (
                    axum::http::StatusCode::OK,
                    format!("Path: {:?}, type: {}, exists: {}", path.canonicalize(), file_type, exists),
                )
            } else {
                (axum::http::StatusCode::NOT_FOUND, format!("Path: {:?}, 404 Not Found", path))
            }
        }
        Err(err) => {
            log::error!("Error: {:?}", err);
            (axum::http::StatusCode::FORBIDDEN, format!("Error: {:?}, 403 Forbidden", path.canonicalize()))
        }
    }
}
