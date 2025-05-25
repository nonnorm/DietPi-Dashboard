use http_body_util::Full;
use hyper::{
    StatusCode,
    body::Bytes,
    header::{self, HeaderName, HeaderValue},
    http::response::Builder as ResponseBuilder,
};

pub struct ServerResponse {
    builder: ResponseBuilder,
    body: Bytes,
}
pub type BuiltResponse = hyper::Response<Full<Bytes>>;

pub enum RedirectType {
    Permanent,
    SeeOther,
}

impl ServerResponse {
    pub fn new() -> Self {
        Self {
            builder: ResponseBuilder::new(),
            body: Bytes::new(),
        }
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.builder = self.builder.status(status);
        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Into<hyper::http::Error>,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        self.builder = self.builder.header(key, value);
        self
    }

    pub fn redirect(self, typ: RedirectType, path: &str) -> Self {
        let status = match typ {
            RedirectType::Permanent => StatusCode::PERMANENT_REDIRECT,
            RedirectType::SeeOther => StatusCode::SEE_OTHER,
        };

        self.status(status).header(header::LOCATION, path)
    }

    pub fn body<T: Into<Bytes>>(mut self, body: T) -> Self {
        self.body = body.into();
        self
    }

    pub fn build(self) -> BuiltResponse {
        let body = Full::from(self.body);

        self.builder.body(body).unwrap()
    }
}
