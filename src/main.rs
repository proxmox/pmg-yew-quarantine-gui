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

use percent_encoding::percent_decode_str;

use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew_router::{HashRouter, Routable, Switch};

use pwt::widget::ThemeLoader;
use pwt::state::{SharedState, SharedStateObserver};
use pwt::touch::PageStack;

use proxmox_yew_comp::{http_login, http_set_auth};
use proxmox_yew_comp::{LoginInfo, ProxmoxProduct};

//http://192.168.3.106:8080/quarantine?ticket=PMGQUAR%253Adietmar%2540proxmox.com%253A644AF198%253A%253AFU%252BowV2YQZxA%252FzzmL16tNoJj0VjZ11aHl4BW7DZsPT9rqFaot2It5ffZdz5Kduclsb4AljhP8Lkmc1qfuqNHxsH%252BKdRgT0hHa8wHL6%252FHbs%252B9OSvtalmh9BCOIpr29V0iA7TLWCUTT1SnBJAKgvu3rk%252BpyenCw4g%252BbdWr6saBbNGKXhNzdX1onN2L2NzoSO9nOhBU9ITXETPncAD0BD9VSsllO112S1857U9RFmw%252B%252B6bk1bRBdnbmsukhoI1XzaPAqoeQ9vgo5FeVKWOWnXnbayhZ84s7xOvozVTBTkwIZ%252FNPEN3OxpRDxxvSaJEnURQc2RM0vcMTjssnw4O0yrgDzg%253D%253D

#[derive(Clone, PartialEq)]
pub struct ReloadController {
    pub state: SharedState<usize>,
}

impl ReloadController {
    pub fn new() -> Self {
        Self {
            state: SharedState::new(0),
        }
    }

    pub fn reload(&self) {
        let mut guard = self.state.write();
        **guard = **guard + 1;
    }

    pub fn add_listener(&self, cb: impl IntoEventCallback<ReloadController>) -> SharedStateObserver<usize> {
        let cb = cb.into_event_callback();
        let me = self.clone();
        self.state.add_listener(move |_| {
            if let Some(cb) = &cb {
                cb.emit(me.clone());
            }
        })
    }
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

fn switch(routes: Route, reload_controller: ReloadController) -> Html {
    let stack = match routes {
        Route::SpamList => {
            vec![PageSpamList::new(reload_controller).into()]
        }
        Route::ViewMail { id } => {
            vec![
                PageSpamList::new(reload_controller.clone()).into(),
                PageMailView::new(reload_controller, id).into(),
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
    reload_controller: ReloadController,
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
        Self {
            login_info,
            reload_controller: ReloadController::new(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let reload_controller = self.reload_controller.clone();
        let render = move |routes: Route| {
            switch(routes, reload_controller.clone())
        };
        ThemeLoader::new(html! {
            <div class="pwt-viewport">
                <HashRouter> // fixme:  basename="/quarantine/">
                    <Switch<Route> {render} />
                </HashRouter>
            </div>
        })
        .into()
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
    proxmox_yew_comp::http_setup(proxmox_yew_comp::ProxmoxProduct::PMG);

    pwt::props::set_http_get_method(
        |url| async move { proxmox_yew_comp::http_get(&url, None).await },
    );

    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<PmgQuarantineApp>::new().render();
}
