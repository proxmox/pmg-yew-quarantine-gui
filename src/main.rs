mod top_nav_bar;
pub use top_nav_bar::TopNavBar;

use log::Log;
use yew::prelude::*;
use yew_router::HashRouter;

use pwt::prelude::*;
use pwt::widget::{Column, Dialog, ThemeLoader};

use proxmox_yew_comp::HttpClient;
use proxmox_yew_comp::LoginInfo;
use proxmox_yew_comp::LoginPanel;
use proxmox_yew_comp::http_login;

enum Msg {
    Login(LoginInfo),
    Logout,
}

struct PmgQuarantineApp {
    login_info: Option<LoginInfo>,
}

impl PmgQuarantineApp {
    fn ticket_login(ctx: &Context<Self>) {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            //https://proxmox-new.maurer-it.com:8006/quarantine?ticket=PMGQUAR%253Adietmar%2540proxmox.com%253A640BB7AD%253A%253ALOPdDlq1oUdi3XxeXuvxU5wPBf4P15CMWjuKIYAX03AXaZIXa%252BGsbvolOn%252BrJ8zTDm7YY1aXfpcTnCIeLuCDLMbDa5qUGuQ2W30Q1xYUuTqY%252Bi8npc6%252BRLib%252FVCbk1a7hB6S5b4k0L7WQZEWefMJgXc6DcVovKRVBBZQ90Fs9gHXxuCK9yS1D4qKoBmMsJipwjnYbuvjZFmRU4m%252BNfu6hwIju%252BOzJNhFq0g1eXV63H8lWjLyCwlb%252BPq%252BZKtvZ5XxPJS9T8Pv%252FAgzCPsdre56oK1FJ%252Byd7EOZDpLkMpLg3pHSr97Ezze5m3R3sfI7Ye4ZZ0NhvbDZhCcGc3XX9wbtlg%253D%253D&cselect=C1R67412T90485159&date=2023-03-10
            let username = "dietmar";
            let realm = "proxmox.com";
            let ticket = "PMGQUAR:dietmar@proxmox.com:640BB7AD::LOPdDlq1oUdi3XxeXuvxU5wPBf4P15CMWjuKIYAX03AXaZIXa+GsbvolOn+rJ8zTDm7YY1aXfpcTnCIeLuCDLMbDa5qUGuQ2W30Q1xYUuTqY+i8npc6+RLib/VCbk1a7hB6S5b4k0L7WQZEWefMJgXc6DcVovKRVBBZQ90Fs9gHXxuCK9yS1D4qKoBmMsJipwjnYbuvjZFmRU4m+Nfu6hwIju+OzJNhFq0g1eXV63H8lWjLyCwlb+Pq+ZKtvZ5XxPJS9T8Pv/AgzCPsdre56oK1FJ+yd7EOZDpLkMpLg3pHSr97Ezze5m3R3sfI7Ye4ZZ0NhvbDZhCcGc3XX9wbtlg==";

            match http_login(username, ticket, realm).await {
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
        let login_info = LoginInfo::from_cookie();
        if let Some(info) = &login_info {
            log::info!("GOT COOKIE");
            proxmox_yew_comp::http_set_auth(info.clone());
        } else {
            log::info!("USE TICKET");
            Self::ticket_login(ctx);
        }
        Self { login_info }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Logout => {
                //log::info!("CLEAR COOKIE");
                proxmox_yew_comp::http_clear_auth();
                self.login_info = None;
                true
            }
            Msg::Login(info) => {
                log::info!("GOT LOGIN");
                self.login_info = Some(info);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onlogin = ctx.link().callback(|info| Msg::Login(info));

        let body = Column::new().class("pwt-viewport")
            .with_child(TopNavBar::new())
            .with_child("TEST PMG");

        ThemeLoader::new(body).into()
    }
}

#[function_component]
fn Scafold() -> Html {
    html! { <HashRouter><PmgQuarantineApp/></HashRouter>}
}

fn main() {
    pwt::props::set_http_get_method(
        |url| async move { proxmox_yew_comp::http_get(&url, None).await },
    );

    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<Scafold>::new().render();
}
