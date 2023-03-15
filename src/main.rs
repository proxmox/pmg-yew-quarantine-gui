mod top_nav_bar;
pub use top_nav_bar::TopNavBar;

mod spam_list;
pub use spam_list::SpamList;

use log::Log;
use percent_encoding::percent_decode_str;

use yew::prelude::*;
use yew_router::HashRouter;

use pwt::prelude::*;
use pwt::widget::{Column, Container, Dialog, ThemeLoader};
use pwt::touch::{Fab, FabMenu, FabMenuAlign};

use proxmox_yew_comp::http_login;
use proxmox_yew_comp::{LoginInfo, LoginPanel, ProxmoxProduct};

enum Msg {
    Preview(String),
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
            let username = "dietmar@proxmox.com";
            let ticket  = percent_decode_str("PMGQUAR%253Adietmar%2540proxmox.com%253A640FAC23%253A%253AQNLhQC%252BULfQrAbIshAdFiPvO7EM%252B6uWYd4Ih%252Fm44OycS6JpRN4w%252FJMQji5%252BwTnTDyQfOfnbTRZujOJEMJLNC6a8r%252F0PklyNDNdubeMLRZffYpTtzSaZ%252FiMs78%252FT6RYz73QrG4Wng%252BjW3cPn%252BrxQ5zIDUJn28oIIX60ajGeXxYv3%252BZdBMqr8%252B0T8EplrwJT%252F6YdGH44%252B%252FlPZo8pXqYuVp2Pl8RwHUB0QIhWy2BW9kvVrM%252FxqG2Odl7YPjSYOAK148ARYSUFfUUvoM3x39TVdBBwuGySgXfv9xgFJsPri0u%252FSOnjFZFYTbp9124Lx%252BTcWCe9CCpowvVOdW3A5vihy4FA%253D%253D")
                     .decode_utf8_lossy();
            let ticket = percent_decode_str(&ticket).decode_utf8_lossy();

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
        let login_info = LoginInfo::from_cookie(ProxmoxProduct::PMG);
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
            Msg::Preview(id) => {
                log::info!("Preview {id}");
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onlogin = ctx.link().callback(|info| Msg::Login(info));

        let content = match &self.login_info {
            Some(info) => {
                SpamList::new()
                    .on_preview(ctx.link().callback(|id| Msg::Preview(id)))
                    .into()
            }
            None => {
                pwt::widget::error_message("Please login first.", "")
            }
        };

        let fab = Container::new()
            .class("pwt-position-fixed")
            .class("pwt-right-2 pwt-bottom-4")
            .with_child(
                FabMenu::new()
                    .align(FabMenuAlign::End)
                    .main_button_class("pwt-scheme-primary")
                    .main_button_class("pwt-fab-small")
                    .with_child(
                        Fab::new("fa fa-times").text("Blacklist")
                    )
                    .with_child(
                        Fab::new("fa fa-check").text("Whitelist")
                    )
                    .with_child(
                        Fab::new("fa fa-trash").text("Delete")
                    )
                    .with_child(
                        Fab::new("fa fa-paper-plane").text("Deliver")
                    )
             );
        let body = Column::new()
            .class("pwt-viewport")
            .with_child(TopNavBar::new())
            .with_child(content)
            .with_child(fab);

        ThemeLoader::new(body).into()
    }
}

#[function_component]
fn Scafold() -> Html {
    html! { <HashRouter><PmgQuarantineApp/></HashRouter>}
}

fn main() {
    proxmox_yew_comp::http_setup(proxmox_yew_comp::ProxmoxProduct::PMG);

    pwt::props::set_http_get_method(
        |url| async move { proxmox_yew_comp::http_get(&url, None).await },
    );

    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<Scafold>::new().render();
}
