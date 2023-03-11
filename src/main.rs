use yew::prelude::*;
use yew_router::HashRouter;

use pwt::prelude::*;
use pwt::widget::{Column, Dialog, ThemeLoader};

use proxmox_yew_comp::LoginInfo;
use proxmox_yew_comp::LoginPanel;

enum Msg {
    Login(LoginInfo),
    Logout,
}

struct PmgQuarantineApp {
    login_info: Option<LoginInfo>,
}

impl Component for PmgQuarantineApp {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let login_info = LoginInfo::from_cookie();
        if let Some(info) = &login_info {
            //log::info!("GOT COOKIE");
            proxmox_yew_comp::http_set_auth(info.clone());
        }
        Self {
            login_info,
        }
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
                //log::info!("LOGIN");
                self.login_info = Some(info);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let onlogin = ctx.link().callback(|info| Msg::Login(info));

        let body = Column::new()
            .class("pwt-viewport")
            .with_child("TEST PMG");

        ThemeLoader::new(body).into()
    }
}

#[function_component]
fn Scafold() -> Html {
    html!{ <HashRouter><PmgQuarantineApp/></HashRouter>}
}

fn main() {

    pwt::props::set_http_get_method(|url| async move {
        proxmox_yew_comp::http_get(&url, None).await
    });

    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<Scafold>::new().render();
}
