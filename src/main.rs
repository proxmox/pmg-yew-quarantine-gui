mod page_stack;
pub use page_stack::PageStack;

mod top_nav_bar;
pub use top_nav_bar::TopNavBar;

mod spam_list;
pub use spam_list::SpamList;

mod page_mail_view;
pub use page_mail_view::PageMailView;

mod page_spam_list;
pub use page_spam_list::PageSpamList;

mod page_not_found;
pub use page_not_found::PageNotFound;

use log::Log;
use percent_encoding::percent_decode_str;

use yew::prelude::*;
use yew_router::scope_ext::RouterScopeExt;
use yew_router::{HashRouter, Routable, Switch};

use pwt::prelude::*;
use pwt::touch::{Fab, FabMenu, FabMenuAlign};
use pwt::widget::{Column, Container, Dialog, ThemeLoader};

use proxmox_yew_comp::{http_login, http_set_auth};
use proxmox_yew_comp::{LoginInfo, LoginPanel, ProxmoxProduct};

//http://192.168.3.106:8080/quarantine?ticket=PMGQUAR%253Adietmar%2540proxmox.com%253A6413A0A7%253A%253A5nZ1NaZiff2WnBwics9sFU6Q2Jj%252BUzhigel85zZt8ui9YkLWSJJ%252F5a1XJ71b9rtU0YwIVp7Nnk3PeHuulANqVaMQSSDELP1qGGj8f8Orj9ybDWXWi5JefM6%252BmE%252Fksvl6k%252F0ehrI1%252Blgd9kTSi6%252B1Fe8QxuPA5ZkIprovs1r6qb8u5903gclJ59AirOntGYj6LtKKbXAKc%252BL13N2b9tgF02vKRrjxObrviAZzJQIS95rl22oooHXcZfHWFonpVgBkXe3AAaboNrqbxBkmVplnV8xbdOVPUpUMnUNLlz3fJvmRdkQCSc3k5v7jhWk8vAEkvwg%252FRjtENBDt1A%252FhkClQlA%253D%253D

use std::sync::atomic::{AtomicUsize, Ordering};
static CHANGE_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn record_data_change() {
    CHANGE_COUNTER.fetch_add(1, Ordering::SeqCst);
}

pub enum Msg {
    Login(LoginInfo),
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
fn switch(routes: Route) -> Html {
    let stack = match routes {
        Route::SpamList => {
            vec![PageSpamList::new(CHANGE_COUNTER.load(Ordering::SeqCst)).into()]
        }
        Route::ViewMail { id } => {
            vec![
                PageSpamList::new(CHANGE_COUNTER.load(Ordering::SeqCst)).into(),
                PageMailView::new(id).into(),
            ]
        }
        Route::NotFound => {
            vec![html! { <PageNotFound/> }]
        }
    };

    PageStack::new(stack).into()
}

struct PmgQuarantineApp {
    login_info: Option<LoginInfo>,
}

impl PmgQuarantineApp {
    fn ticket_login(ctx: &Context<Self>, username: String, ticket: String) {
        let link = ctx.link().clone();

        wasm_bindgen_futures::spawn_local(async move {
            match http_login(username, ticket, "quarantine").await {
                Ok(info) => {
                    link.send_message(Msg::Login(info));
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
        let login_info = LoginInfo::from_cookie(ProxmoxProduct::PMG);
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

    fn view(&self, ctx: &Context<Self>) -> Html {
        ThemeLoader::new(html! {
            <HashRouter> // fixme:  basename="/quarantine/">
                <Switch<Route> render={switch} />
            </HashRouter>
        })
        .into()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Login(info) => {
                self.login_info = Some(info);
                let document = web_sys::window().unwrap().document().unwrap();
                let location = document.location().unwrap();
                location.replace("/");
            }
        }
        true
    }
}

fn main() {
    proxmox_yew_comp::http_setup(proxmox_yew_comp::ProxmoxProduct::PMG);

    pwt::props::set_http_get_method(
        |url| async move { proxmox_yew_comp::http_get(&url, None).await },
    );

    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<PmgQuarantineApp>::new().render();
}
