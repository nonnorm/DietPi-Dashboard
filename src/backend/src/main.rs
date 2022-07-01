#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)]
use crate::shared::CONFIG;
use anyhow::Context;
use ring::digest;
use std::{net::IpAddr, str::FromStr};
use tracing_subscriber::layer::{Layer, SubscriberExt};
use warp::Filter;
#[cfg(feature = "frontend")]
use warp::{http::header, Reply};

mod config;
mod page_handlers;
mod shared;
mod socket_handlers;
mod systemdata;

struct BeQuietWarp {
    log_level: tracing_subscriber::filter::LevelFilter,
}

impl<S: tracing::Subscriber> Layer<S> for BeQuietWarp {
    fn enabled(
        &self,
        metadata: &tracing::Metadata<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        !(metadata.target() == "warp::filters::trace" && *metadata.level() >= self.log_level)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "frontend")]
    const DIR: include_dir::Dir = include_dir::include_dir!("dist");

    {
        let log_level = tracing_subscriber::filter::LevelFilter::from_str(&CONFIG.log_level)
            .context("Couldn't parse log level")?;
        tracing::subscriber::set_global_default(
            tracing_subscriber::FmtSubscriber::builder()
                .with_max_level(log_level)
                .with_timer(tracing_subscriber::fmt::time::uptime())
                .finish()
                .with(BeQuietWarp { log_level }),
        )
        .context("Couldn't init logger")?;
    }

    #[cfg(feature = "frontend")]
    let mut headers = header::HeaderMap::new();
    #[cfg(feature = "frontend")]
    {
        headers.insert(
            header::X_CONTENT_TYPE_OPTIONS,
            header::HeaderValue::from_static("nosniff"),
        );
        headers.insert(
            header::X_FRAME_OPTIONS,
            header::HeaderValue::from_static("sameorigin"),
        );
        headers.insert("X-Robots-Tag", header::HeaderValue::from_static("none"));
        headers.insert(
            "X-Permitted-Cross-Domain_Policies",
            header::HeaderValue::from_static("none"),
        );
        headers.insert(
            header::REFERRER_POLICY,
            header::HeaderValue::from_static("no-referrer"),
        );
        headers.insert("Content-Security-Policy", header::HeaderValue::from_static("default-src 'self'; font-src 'self'; img-src 'self' blob:; script-src 'self'; style-src 'unsafe-inline' 'self'; connect-src * ws:;"));
        #[cfg(feature = "compression")]
        headers.insert(
            header::CONTENT_ENCODING,
            header::HeaderValue::from_static("gzip"),
        );
    }

    #[cfg(feature = "frontend")]
    let favicon_route = warp::path("favicon.png").map(|| {
        let _guard = tracing::info_span!("favicon_route");
        warp::reply::with_header(
            handle_error!(
                DIR.get_file("favicon.png").context("Couldn't get favicon"),
                return warp::reply::with_status(
                    "Couldn't get favicon",
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR
                )
                .into_response()
            )
            .contents(),
            "content-type",
            "image/png",
        )
        .into_response()
    });

    #[cfg(feature = "frontend")]
    let assets_route = warp::path("assets")
        .and(warp::path::param())
        .map(|path: String| {
            let _guard = tracing::info_span!("asset_route").entered();
            let ext = path.rsplit('.').next().unwrap_or("plain");
            #[allow(unused_mut)]
            // Mute warning, variable is mut because it's used with the compression feature
            let mut reply = warp::reply::with_header(
                match DIR.get_file(format!("assets/{}", path)) {
                    Some(file) => file.contents(),
                    None => {
                        tracing::warn!("Couldn't get asset {}", path);
                        return warp::reply::with_status(
                            "Asset not found",
                            warp::http::StatusCode::NOT_FOUND,
                        )
                        .into_response();
                    }
                },
                header::CONTENT_TYPE,
                if ext == "js" {
                    "text/javascript".to_string()
                } else if ext == "svg" {
                    "image/svg+xml".to_string()
                } else if ext == "png" {
                    "image/png".to_string()
                } else {
                    format!("text/{}", ext)
                },
            )
            .into_response();

            #[cfg(feature = "compression")]
            if ext != "png" {
                reply.headers_mut().insert(
                    header::CONTENT_ENCODING,
                    header::HeaderValue::from_static("gzip"),
                );
            };

            reply
        });

    let login_route = warp::path("login")
        .and(warp::post())
        .and(warp::body::bytes())
        .map(|pass: warp::hyper::body::Bytes| {
            let _guard = tracing::info_span!("login_route").entered();
            let token: String;
            if CONFIG.pass {
                let shasum = digest::digest(&digest::SHA512, &pass)
                    .as_ref()
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                if shasum == CONFIG.hash {
                    let timestamp = jsonwebtoken::get_current_timestamp();

                    let claims = crate::shared::JWTClaims {
                        iss: "DietPi Dashboard".to_string(),
                        iat: timestamp,
                        exp: timestamp + CONFIG.expiry,
                    };

                    token = handle_error!(
                        jsonwebtoken::encode(
                            &jsonwebtoken::Header::default(),
                            &claims,
                            &jsonwebtoken::EncodingKey::from_secret(CONFIG.secret.as_ref()),
                        )
                        .context("Error creating login token"),
                        return warp::reply::with_status(
                            "Error creating login token".to_string(),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        )
                    );

                    return warp::reply::with_status(token, warp::http::StatusCode::OK);
                }
                return warp::reply::with_status(
                    "Unauthorized".to_string(),
                    warp::http::StatusCode::UNAUTHORIZED,
                );
            }
            warp::reply::with_status("No login needed".to_string(), warp::http::StatusCode::OK)
        })
        .with(warp::reply::with::header(
            "Access-Control-Allow-Origin",
            "*",
        ));

    // The spans for these are covered in the handlers
    let terminal_route = warp::path("ws")
        .and(warp::path("term"))
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(socket_handlers::term_handler));

    let socket_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(socket_handlers::socket_handler));

    let file_route = warp::path("ws")
        .and(warp::path("file"))
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(socket_handlers::file_handler));

    #[cfg(feature = "frontend")]
    let main_route = warp::any()
        .map(|| {
            let _guard = tracing::info_span!("main_route").entered();
            let file = handle_error!(
                DIR.get_file("index.html")
                    .context("Couldn't get main HTML file"),
                return warp::reply::with_status(
                    "Couldn't get main HTML file",
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR
                )
                .into_response()
            )
            .contents();
            warp::reply::html(file).into_response()
        })
        .with(warp::reply::with::headers(headers));

    #[cfg(feature = "frontend")]
    let page_routes = favicon_route.or(assets_route).or(main_route);

    let socket_routes = terminal_route.or(file_route).or(socket_route);

    let routes = socket_routes.or(login_route);
    #[cfg(feature = "frontend")]
    let routes = routes.or(page_routes);
    let routes = routes.with(warp::trace::trace(|info| {
        let remote_addr = info
            .remote_addr()
            .unwrap_or_else(|| std::net::SocketAddr::from((std::net::Ipv4Addr::UNSPECIFIED, 0)))
            .ip();
        let span = tracing::info_span!("request", %remote_addr);
        span.in_scope(|| {
            tracing::info!("Request to {}", info.path());
            tracing::debug!(
                "by {}, using {} {:?}",
                info.user_agent().unwrap_or("unknown"),
                remote_addr,
                info.version(),
            );
        });
        span
    }));

    let addr = IpAddr::from([0; 8]);

    if CONFIG.tls {
        warp::serve(routes)
            .tls()
            .cert_path(&CONFIG.cert)
            .key_path(&CONFIG.key)
            .run((addr, CONFIG.port))
            .await;
    } else {
        warp::serve(routes).run((addr, CONFIG.port)).await;
    }

    Ok(())
}
