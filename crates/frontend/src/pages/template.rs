use hyper::header;
use maud::{DOCTYPE, Markup, Render, html};

use crate::http::{
    request::{BackendData, ServerRequest},
    response::ServerResponse,
};

macro_rules! send_req {
    ($req:expr, $variant:ident $(($data:expr))?) => {{
        use proto::{backend::ResponseBackendMessage, frontend::RequestFrontendMessage};

        $req.send_backend_req(RequestFrontendMessage::$variant $(($data))?)
            .await
            .map(|resp| match resp {
                ResponseBackendMessage::$variant(resp) => resp,
                _ => unreachable!(),
            })
    }};
}

pub(crate) use send_req;

macro_rules! send_act {
    ($req:expr, $variant:ident $(($data:expr))?) => {{
        use proto::frontend::ActionFrontendMessage;

        $req.send_backend_action(ActionFrontendMessage::$variant $(($data))?).await
    }};
}

pub(crate) use send_act;

fn header(req: &ServerRequest) -> Result<Markup, ServerResponse> {
    let BackendData {
        backend_list,
        current_backend,
    } = req.extract_backends()?;

    Ok(html! {
        header {
            button
                aria-controls="nav"
                nm-bind="onclick: () => navOpen = !navOpen, ariaExpanded: () => navOpen"
            {
                (Icon::new("fa6-solid-bars").size(48))
            }

            label {
                "Backend: "
                select
                    onchange="document.cookie = `backend=${this.value}; MaxAge=999999999`; window.location.reload()"
                {
                    @for backend in backend_list {
                        @let is_current_backend = backend.0 == current_backend.addr;
                        option value=(backend.0) selected[is_current_backend] {
                            (backend.1) " (" (backend.0) ")"
                        }
                    }
                }
            }

            button .msg-btn
                aria-controls="msgs"
                nm-bind="onclick: () => msgsOpen = !msgsOpen, ariaExpanded: () => msgsOpen"
            {
                (Icon::new("fa6-solid-envelope"))
                span .notifier nm-bind="hidden: () => !newMsg" { (Icon::new("fa6-solid-circle").size(12)) }
            }

            span nm-data="isDark: localStorage.getItem('darkMode') === 'true'" {
                meta
                    name="color-scheme"
                    nm-bind="content: () => isDark ? 'dark' : 'light'"
                {}
                button nm-bind="
                    onclick: () => {
                        isDark = !isDark;
                        localStorage.setItem('darkMode', isDark);
                    }
                " {
                    span nm-bind="hidden: () => isDark" {
                        (Icon::new("fa6-solid-sun"))
                    }
                    span nm-bind="hidden: () => !isDark" {
                        (Icon::new("fa6-solid-moon"))
                    }
                }
            }
        }
        #msgs {
            ul {
                li nm-bind={"textContent: async () => {
                    const msg = await getUpdateMessage('"(config::APP_VERSION)"');
                    newMsg = !!msg;
                    return msg;
                }"} {}
                @if let Some(update) = current_backend.update {
                    li nm-bind="_: () => newMsg = true" { "DietPi Update Available: " (update) }
                }
            }
        }
    })
}

fn nav() -> Markup {
    html! {
        nav #nav {
            a href="/system" {
                (Icon::new("fa6-solid-gauge"))
                "System"
            }
            a href="/process" {
                (Icon::new("fa6-solid-microchip"))
                "Processes"
            }
            a href="/software" {
                (Icon::new("fa6-solid-database"))
                "Software"
            }
            a href="/service" {
                (Icon::new("fa6-solid-list"))
                "Services"
            }
            a href="/management" {
                (Icon::new("fa6-solid-user"))
                "Management"
            }
            a href="/terminal" {
                (Icon::new("fa6-solid-terminal"))
                "Terminal"
            }
            a href="/browser" {
                (Icon::new("fa6-solid-folder"))
                "File Browser"
            }
        }
    }
}

fn footer() -> Markup {
    html! {
        footer {
            "DietPi Dashboard v" (config::APP_VERSION) " by ravenclaw900"
            a href="https://github.com/ravenclaw900/DietPi-Dashboard" target="_blank" {
                (Icon::new("cib-github").size(32))
            }
        }
    }
}

pub fn template(req: &ServerRequest, content: Markup) -> Result<ServerResponse, ServerResponse> {
    let page = if req.is_fixi() {
        content
    } else {
        html! {
            (DOCTYPE)
            html lang="en" {
                head {
                    meta charset="UTF-8";
                    meta name="viewport" content="width=device-width, initial-scale=1";

                    title { "DietPi Dashboard" }

                    link rel="stylesheet" href="/static/main.css";
                }
                body
                    nm-data="navOpen: true, msgsOpen: false, newMsg: false,"
                    nm-bind="className: () => `${navOpen ? '' : 'nav-closed'} ${msgsOpen ? 'msgs-open' : ''}`"
                {
                    h1 { "DietPi Dashboard" }

                    (header(req)?)

                    (nav())

                    main nm-data {
                        p nm-bind="textContent: () => nmError" {}

                        (content)
                    }

                    (footer())

                    script src="/static/main.js" {}
                }
            }
        }
    };

    Ok(ServerResponse::new()
        .header(header::CONTENT_TYPE, "text/html;charset=UTF-8")
        .body(page.into_string()))
}

pub struct Icon {
    name: &'static str,
    size: u8,
}

impl Icon {
    pub fn new(name: &'static str) -> Self {
        Self { name, size: 24 }
    }

    pub fn size(mut self, size: u8) -> Self {
        self.size = size;
        self
    }
}

impl Render for Icon {
    fn render(&self) -> Markup {
        html! {
            svg width=(self.size) height=(self.size) {
                use href={"/static/icons.svg#" (self.name)} {}
            }
        }
    }
}
