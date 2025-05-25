use hyper::header;
use maud::html;
use ring::digest::{SHA512, digest};
use serde::Deserialize;

use crate::http::{
    request::ServerRequest,
    response::{RedirectType, ServerResponse},
};

use super::template::template;

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    if !req.config().enable_login {
        return Err(ServerResponse::new().redirect(RedirectType::SeeOther, "/"));
    }

    let content = html! {
        section {
            h2 { "Login" }

            form method="POST" {
                input name="pass" type="password" placeholder="Password" {}
                input type="submit" {}
            }
        }
    };

    template(&req, content)
}

#[derive(Deserialize)]
pub struct LoginForm {
    pass: String,
}

pub async fn form(mut req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    if !req.config().enable_login {
        return Err(ServerResponse::new().redirect(RedirectType::SeeOther, "/"));
    }

    let hash = req.config().hash.clone();
    let form: LoginForm = req.extract_form().await?;

    let form_hash = digest(&SHA512, form.pass.as_bytes());
    let form_hash = data_encoding::HEXLOWER.encode(form_hash.as_ref());

    if form_hash == hash {
        let logins = req.extract_logins();
        let mut logins = logins.get();

        let token = logins.new_token();

        Ok(ServerResponse::new()
            .redirect(RedirectType::SeeOther, "/")
            .header(
                header::SET_COOKIE,
                format!("token={token}; Max-Age=3600; Path=/; HttpOnly"),
            ))
    } else {
        Err(ServerResponse::new().redirect(RedirectType::SeeOther, "/login"))
    }
}
