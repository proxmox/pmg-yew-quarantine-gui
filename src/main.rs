mod spam_list;

pub use spam_list::SpamList;

mod page_mail_view;
pub use page_mail_view::PageMailView;

mod page_spam_list;
pub use page_spam_list::PageSpamList;

mod page_not_found;
pub use page_not_found::PageNotFound;

mod page_login;
pub use page_login::PageLogin;

use anyhow::Error;
use gloo_utils::{document, format::JsValueSerdeExt};
use serde::Deserialize;
use serde_json::{json, Value};
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew_router::Routable;

use pwt::prelude::*;
use pwt::state::LanguageInfo;
use pwt::touch::MaterialApp;

use proxmox_login::Authentication;
use proxmox_yew_comp::{
    authentication_from_cookie, http_post, http_set_auth, register_auth_observer,
    stop_ticket_refresh_loop, AuthObserver, ExistingProduct,
};

// Note: The server provides this data with the template
#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct ServerConfig {
    pub i18nVersion: String,
    pub uiVersion: String,
    pub basePath: String,
}

pub enum Msg {
    Login(Authentication),
    Logout,
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    SpamList,
    #[at("/post/:id")]
    ViewMail { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(route: &str) -> Vec<Html> {
    let routes = Routable::recognize(route).unwrap();

    match routes {
        Route::SpamList => {
            vec![PageSpamList::new().into()]
        }
        Route::ViewMail { id } => {
            vec![PageSpamList::new().into(), PageMailView::new(id).into()]
        }
        Route::NotFound => {
            vec![html! { <PageNotFound/> }]
        }
    }
}

struct PmgQuarantineApp {
    login_info: Option<Authentication>,
    _auth_observer: AuthObserver,
    server_config: Option<ServerConfig>,
}

impl Component for PmgQuarantineApp {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut server_config = None;
        if let Some(window) = web_sys::window() {
            if let Ok(value) = js_sys::Reflect::get(&window, &JsValue::from_str("Proxmox")) {
                if let Ok(config) = JsValueSerdeExt::into_serde::<ServerConfig>(&value) {
                    server_config = Some(config);
                }
            }
        }
        // set auth info from cookie
        let login_info = authentication_from_cookie(&ExistingProduct::PMG);
        if let Some(login_info) = &login_info {
            http_set_auth(login_info.clone());
            if login_info.ticket.to_string().starts_with("PMGQUAR:") {
                stop_ticket_refresh_loop();
            }
        }
        let _auth_observer = register_auth_observer(
            ctx.link()
                .batch_callback(|logout: bool| logout.then_some(Msg::Logout)),
        );
        Self {
            login_info,
            _auth_observer,
            server_config,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link().clone();
        let logged_in = self.login_info.is_some();
        MaterialApp::new(move |path: &str| {
            if logged_in {
                switch(path)
            } else {
                vec![PageLogin::new().on_login(link.callback(Msg::Login)).into()]
            }
        })
        .theme_url_builder({
            let ui_version = self.server_config.as_ref().map(|c| c.uiVersion.clone());
            let base_path = self
                .server_config
                .as_ref()
                .map(|c| c.basePath.clone())
                .unwrap_or("/mobile".into());
            move |theme: &String| {
                let url = format!("{base_path}/css/{}-yew-style.css", theme.to_lowercase());
                if let Some(version) = &ui_version {
                    format!("{url}?v{version}")
                } else {
                    url
                }
            }
        })
        .catalog_url_builder({
            let i18n_version = self.server_config.as_ref().map(|c| c.i18nVersion.clone());
            let base_path = self
                .server_config
                .as_ref()
                .map(|c| c.basePath.clone())
                .unwrap_or("/mobile".into());
            move |lang: &String| {
                let url = format!("{base_path}/i18n/pmg-yew-quarantine-catalog-{lang}.mo");
                if let Some(version) = &i18n_version {
                    format!("{url}?v{version}")
                } else {
                    url
                }
            }
        })
        .into()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Login(info) => {
                self.login_info = Some(info.clone());
                let document = document();
                let location = document.location().unwrap();
                let _ = location.replace(&location.pathname().unwrap());
            }
            Msg::Logout => self.login_info = None,
        }
        true
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum MailAction {
    Deliver,
    Delete,
    Whitelist,
    Blacklist,
}

impl std::fmt::Display for MailAction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            MailAction::Deliver => "deliver",
            MailAction::Delete => "delete",
            MailAction::Whitelist => "whitelist",
            MailAction::Blacklist => "blacklist",
        })
    }
}

pub(crate) async fn mail_action(id: &str, action: MailAction) -> Result<Value, Error> {
    let param = json!({
        "action": action.to_string(),
        "id": id,
    });
    http_post("/quarantine/content", Some(param)).await
}

fn main() {
    proxmox_yew_comp::http_setup(&ExistingProduct::PMG);

    pwt::state::set_available_themes(&["Mobile"]);
    pwt::state::set_available_languages(vec![LanguageInfo::new(
        "en",
        "English",
        gettext_noop("English"),
    )]);
    pwt::props::set_http_get_method(
        |url| async move { proxmox_yew_comp::http_get(&url, None).await },
    );

    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<PmgQuarantineApp>::new().render();
}
