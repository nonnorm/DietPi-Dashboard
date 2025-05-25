use hyper::header;

use super::{request::ServerRequest, response::ServerResponse};

macro_rules! static_file {
    ($name:ident, $path:literal, $mime:literal) => {
        pub async fn $name(_req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
            let file = include_bytes!($path);

            Ok(ServerResponse::new()
                .header(header::CONTENT_TYPE, $mime)
                .header(header::CONTENT_ENCODING, "gzip")
                .body(&file[..]))
        }
    };
}

static_file!(js, "../../dist/main.js", "text/javascript;charset=UTF-8");
static_file!(css, "../../dist/main.css", "text/css;charset=UTF-8");
static_file!(icons, "../../dist/icons.svg", "image/svg+xml");
