mod spam_list;
pub use spam_list::SpamList;

mod page_mail_view;
pub use page_mail_view::PageMailView;

mod page_spam_list;
pub use page_spam_list::PageSpamList;

mod page_not_found;
pub use page_not_found::PageNotFound;

use percent_encoding::percent_decode_str;

use yew::prelude::*;
use yew_router::Routable;

use pwt::prelude::*;
use pwt::state::LanguageInfo;
use pwt::touch::MaterialApp;

use proxmox_login::{Authentication, TicketResult};
use proxmox_yew_comp::{authentication_from_cookie, http_login, http_set_auth, ExistingProduct};

pub enum Msg {
    Login(Authentication),
    //Logout,
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
}

impl PmgQuarantineApp {
    fn ticket_login(ctx: &Context<Self>, username: String, ticket: String) {
        let link = ctx.link().clone();

        wasm_bindgen_futures::spawn_local(async move {
            match http_login(username, ticket, "quarantine").await {
                Ok(TicketResult::Full(info)) => {
                    link.send_message(Msg::Login(info));
                }
                Ok(TicketResult::HttpOnly(info)) => {
                    link.send_message(Msg::Login(info));
                }
                Ok(TicketResult::TfaRequired(_)) => {
                    log::error!("ERROR: TFA required, but not implemenmted");
                }
                Err(err) => {
                    log::error!("ERROR: {:?}", err);
                    //link.send_message(Msg::LoginError(err.to_string()));
                }
            }
        });
    }
}

impl Component for PmgQuarantineApp {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // set auth info from cookie
        let login_info = authentication_from_cookie(&ExistingProduct::PMG);
        if let Some(login_info) = &login_info {
            http_set_auth(login_info.clone());
        }
        // Autologin with quartantine url and ticket
        let document = web_sys::window().unwrap().document().unwrap();
        let location = document.location().unwrap();
        let path = location.pathname().unwrap();
        if path == "/quarantine" {
            let search = location.search().unwrap();
            let param = web_sys::UrlSearchParams::new_with_str(&search).unwrap();
            if let Some(ticket) = param.get("ticket") {
                let ticket = percent_decode_str(&ticket).decode_utf8_lossy();
                if ticket.starts_with("PMGQUAR:") {
                    if let Some(username) = ticket.split(":").nth(1) {
                        Self::ticket_login(ctx, username.to_string(), ticket.to_string());
                    }
                }
            }
        }
        Self { login_info }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        MaterialApp::new(switch).into()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Login(info) => {
                self.login_info = Some(info);
                let document = web_sys::window().unwrap().document().unwrap();
                let location = document.location().unwrap();
                let _ = location.replace("/");
            }
        }
        true
    }
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
