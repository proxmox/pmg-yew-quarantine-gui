use std::rc::Rc;

use gloo_utils::document;
use percent_encoding::percent_decode_str;
use proxmox_login::{Authentication, TicketResult};
use yew::{
    html::IntoEventCallback,
    virtual_dom::{VComp, VNode},
    Callback, Component, Properties,
};

use pwt::{
    css::{AlignItems, JustifyContent},
    props::{ContainerBuilder, CssPaddingBuilder, WidgetBuilder},
    touch::{SnackBar, SnackBarContextExt},
    widget::{Column, Container, Image, Row},
};

use proxmox_yew_comp::{
    http_login, start_ticket_refresh_loop, stop_ticket_refresh_loop, LoginPanel,
};

#[derive(Properties, PartialEq)]
pub struct PageLogin {
    #[prop_or_default]
    on_login: Option<Callback<Authentication>>,
}

pub enum Msg {
    Login(Authentication),
    LoginError(proxmox_client::Error),
}

impl PageLogin {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn on_login(mut self, on_login: impl IntoEventCallback<Authentication>) -> Self {
        self.on_login = on_login.into_event_callback();
        self
    }
}

impl Default for PageLogin {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PmgPageLogin {}

impl PmgPageLogin {
    fn ticket_login(ctx: &yew::Context<Self>, username: String, ticket: String) {
        let link = ctx.link().clone();

        wasm_bindgen_futures::spawn_local(async move {
            start_ticket_refresh_loop();
            match http_login(username, ticket, "quarantine").await {
                Ok(TicketResult::Full(info) | TicketResult::HttpOnly(info)) => {
                    link.send_message(Msg::Login(info));
                }
                Ok(TicketResult::TfaRequired(_)) => {
                    log::error!("ERROR: TFA required, but not implemented");
                }
                Err(err) => {
                    link.send_message(Msg::LoginError(err));
                }
            }
        });
    }
}

impl Component for PmgPageLogin {
    type Message = Msg;
    type Properties = PageLogin;

    fn create(ctx: &yew::Context<Self>) -> Self {
        // Autologin with quartantine url and ticket
        let document = document();
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
                        stop_ticket_refresh_loop();
                    }
                }
            }
        }

        Self {}
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Login(authentication) => {
                if let Some(cb) = &props.on_login {
                    cb.emit(authentication);
                }
            }
            Msg::LoginError(error) => {
                ctx.link()
                    .show_snackbar(SnackBar::new().message(error.to_string()));
            }
        }
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        Column::new()
            .with_child(
                Row::new()
                    .padding(2)
                    .gap(1)
                    .class(AlignItems::Center)
                    .class(JustifyContent::Center)
                    .with_child(
                        Image::new("/proxmox_logo.png").dark_mode_src("/proxmox_logo_white.png"),
                    )
                    .with_child(
                        Container::new()
                            .class("pwt-font-headline-small")
                            .with_child("Mail Gateway"),
                    ),
            )
            .with_child(
                LoginPanel::new()
                    .mobile(true)
                    .domain_path(Some("/access/auth-realm".into()))
                    .on_login(ctx.link().callback(Msg::Login)),
            )
            .into()
    }
}

impl From<PageLogin> for VNode {
    fn from(val: PageLogin) -> Self {
        let comp = VComp::new::<PmgPageLogin>(Rc::new(val), None);
        VNode::from(comp)
    }
}
