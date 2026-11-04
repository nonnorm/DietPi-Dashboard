use hyper::header;
use maud::{DOCTYPE, Markup, PreEscaped, Render, html};

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
            button .nav-toggle
                title="Toggle navigation"
                aria-label="Toggle navigation"
                data-i18n-title="toggle_navigation"
                data-i18n-aria-label="toggle_navigation"
                aria-controls="nav"
                nm-bind="onclick: () => navOpen = !navOpen, ariaExpanded: () => navOpen"
            {
                (Icon::new("fa6-solid-bars").size(30))
            }

            label .backend-switch {
                span data-i18n="backend" { "Backend" }
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

            button .lang-toggle
                type="button"
                data-lang-value="1"
                title="Toggle language"
                aria-label="Toggle language"
                data-i18n-title="toggle_language"
                data-i18n-aria-label="toggle_language"
                onclick="window.__setDashboardLang?.(document.documentElement.dataset.lang === 'zh' ? 'en' : 'zh')"
            { "EN" }

            button .msg-btn
                title="Messages"
                aria-label="Messages"
                data-i18n-title="messages"
                data-i18n-aria-label="messages"
                aria-controls="msgs"
                nm-bind="onclick: () => msgsOpen = !msgsOpen, ariaExpanded: () => msgsOpen"
            {
                (Icon::new("fa6-solid-envelope"))
                span .notifier nm-bind="hidden: () => !newMsg" { (Icon::new("fa6-solid-circle").size(12)) }
            }

            span .theme-switch nm-data="isDark: localStorage.getItem('darkMode') === 'true'" nm-bind="
                oninit: () => {
                    const theme = isDark ? 'dark' : 'light';
                    window.__setDashboardTheme?.(theme);
                }
            " {
                button .theme-toggle
                    title="Toggle theme"
                    aria-label="Toggle theme"
                    data-i18n-title="toggle_theme"
                    data-i18n-aria-label="toggle_theme"
                    nm-bind="
                    onclick: () => {
                        isDark = !isDark;
                        localStorage.setItem('darkMode', isDark);
                        const theme = isDark ? 'dark' : 'light';
                        window.__setDashboardTheme?.(theme);
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
                    li
                        nm-bind="oninit: () => newMsg = true"
                        data-i18n-template="dietpi_update_available"
                        data-version=(update)
                    { "DietPi Update Available: " (update) }
                }
            }
        }
    })
}

fn nav(req: &ServerRequest) -> Markup {
    let current_page = req.path_segments().next().unwrap_or("system");

    html! {
        nav #nav nm-bind="
            onclick: (e) => {
                if (window.matchMedia('(max-width: 980px)').matches && e.target.closest('a')) {
                    navOpen = false;
                }
            }
        " {
            a href="/system" class=(if current_page == "system" { "active" } else { "" }) aria-current=(if current_page == "system" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-gauge"))
                span data-i18n="nav_system" { "System" }
            }
            a href="/process" class=(if current_page == "process" { "active" } else { "" }) aria-current=(if current_page == "process" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-microchip"))
                span data-i18n="nav_processes" { "Processes" }
            }
            a href="/software" class=(if current_page == "software" { "active" } else { "" }) aria-current=(if current_page == "software" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-database"))
                span data-i18n="nav_software" { "Software" }
            }
            a href="/service" class=(if current_page == "service" { "active" } else { "" }) aria-current=(if current_page == "service" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-list"))
                span data-i18n="nav_services" { "Services" }
            }
            a href="/management" class=(if current_page == "management" { "active" } else { "" }) aria-current=(if current_page == "management" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-user"))
                span data-i18n="nav_management" { "Management" }
            }
            a href="/terminal" class=(if current_page == "terminal" { "active" } else { "" }) aria-current=(if current_page == "terminal" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-terminal"))
                span data-i18n="nav_terminal" { "Terminal" }
            }
            a href="/browser" class=(if current_page == "browser" { "active" } else { "" }) aria-current=(if current_page == "browser" { "page" } else { "false" }) {
                (Icon::new("fa6-solid-folder"))
                span data-i18n="nav_file_browser" { "File Browser" }
            }
        }
    }
}

fn footer() -> Markup {
    html! {
        footer {
            p .footer-meta {
                span data-i18n="footer_product" { "DietPi Dashboard" }
                span { " v" (config::APP_VERSION) }
                span data-i18n="footer_by" { "by" }
                a href="https://github.com/nonnorm" target="_blank" rel="noopener noreferrer" { "ravenclaw900" }
                span .footer-sep { "·" }
                span data-i18n="footer_design_by" { "WebUI Design by" }
                a href="https://github.com/CJackHwang" target="_blank" rel="noopener noreferrer" { "CJackHwang" }
            }
            a .footer-repo href="https://github.com/nonnorm/DietPi-Dashboard" target="_blank" rel="noopener noreferrer" title="DietPi-Dashboard Repository" data-i18n-title="footer_repo_title" {
                (Icon::new("cib-github").size(32))
            }
        }
    }
}

pub fn template(
    req: &ServerRequest,
    content: Markup,
    persistent_data: &str,
) -> Result<ServerResponse, ServerResponse> {
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

                    script {
                        (PreEscaped(r#"
                            (() => {
                                const root = document.documentElement;
                                const normalizeLang = (lang) =>
                                    String(lang || '').toLowerCase().startsWith('zh') ? 'zh' : 'en';

                                const setTheme = (theme) => {
                                    root.dataset.theme = theme;
                                    root.style.colorScheme = theme;

                                    let meta = document.querySelector('meta[name="color-scheme"]');
                                    if (!meta) {
                                        meta = document.createElement('meta');
                                        meta.name = 'color-scheme';
                                        document.head.append(meta);
                                    }
                                    meta.content = theme;
                                };

                                const setLang = (lang) => {
                                    const next = normalizeLang(lang);
                                    root.dataset.lang = next;
                                    root.lang = next === 'zh' ? 'zh-CN' : 'en';
                                    try {
                                        localStorage.setItem('dashboardLang', next);
                                    } catch (_) {}
                                    window.__applyDashboardI18n?.(next);
                                };

                                window.__setDashboardTheme = setTheme;
                                window.__setDashboardLang = setLang;
                                let isDark = false;
                                let lang = 'en';
                                try {
                                    isDark = localStorage.getItem('darkMode') === 'true';
                                    lang = normalizeLang(
                                        localStorage.getItem('dashboardLang') || navigator.language
                                    );
                                } catch (_) {}
                                setTheme(isDark ? 'dark' : 'light');
                                setLang(lang);
                            })();
                        "#))
                    }

                    link rel="icon" href="/favicon.svg" type="image/svg+xml";
                    link rel="stylesheet" href="/static/main.css";
                }
                body
                    nm-data="navOpen: window.matchMedia('(min-width: 981px)').matches, msgsOpen: false, newMsg: false,"
                    nm-bind="
                        oninit: () => {
                            const media = window.matchMedia('(max-width: 980px)');
                            media.addEventListener('change', () => {
                                navOpen = !media.matches;
                                msgsOpen = false;
                            });
                        },
                        'class.nav-closed': () => !navOpen,
                        'class.nav-open': () => navOpen,
                        'class.msgs-open': () => msgsOpen
                    "
                {
                    h1 data-i18n="app_name" { "DietPi Dashboard" }

                    (header(req)?)

                    (nav(req))
                    button #nav-overlay type="button" aria-label="Close navigation" data-i18n-aria-label="close_navigation" nm-bind="
                        hidden: () => !navOpen,
                        onclick: () => navOpen = false
                    " {}

                    main nm-data=(persistent_data) {
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
