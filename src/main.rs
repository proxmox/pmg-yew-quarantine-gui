mod top_nav_bar;
pub use top_nav_bar::TopNavBar;

mod spam_list;
pub use spam_list::SpamList;

use log::Log;
use percent_encoding::percent_decode_str;

use yew::prelude::*;
use yew_router::HashRouter;

use pwt::prelude::*;
use pwt::widget::{Column, Dialog, ThemeLoader};

use proxmox_yew_comp::http_login;
use proxmox_yew_comp::LoginInfo;
use proxmox_yew_comp::LoginPanel;
use proxmox_yew_comp::ProxmoxProduct;

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
            let username = "dietmar@proxmox.com";
            let ticket  = percent_decode_str("PMGQUAR%253Adietmar%2540proxmox.com%253A640D0922%253A%253ARHD4wNz7F%252BVWFeOTlCWGJLi43oR5mJWw3HrDr7EvBb9azzRMAdSrcJztKLj%252BUfPjnG7R%252BQkPJyrlDkyE3pW8fInjjVrn0bNuDiVN7Y9GN1OUsXsC0noDCUcVhTU1TRQv2XLp%252Bur8Sz8gGTspKib%252F4StwZxltKa78RZyOqv716Lo5x6o4MbuYHcXZfIYcRtMZlgJEUhE710UNHQ0YU5dv1F7uGKw0koI7QaWYv00Di7cmsm1VJCsO5XYyFLVe8dOv8uuHUMf8Mt4Etw7BOQjzOWYJhXc%252Bkf4wb8nHFME%252BisFbUhf4qN9RN9nGAmTTut%252F6i1tdUwGnHtM4tbTVegJ6KA%253D%253D")
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onlogin = ctx.link().callback(|info| Msg::Login(info));

        let content = match &self.login_info {
            Some(info) => {
                SpamList::new()
                    .into()
            }
            None => {
                pwt::widget::error_message("Please login first.", "")
            }
        };

        let body = Column::new()
            .class("pwt-viewport")
            .with_child(TopNavBar::new())
            .with_child(content);

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
