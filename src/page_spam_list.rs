use log::Log;
use percent_encoding::percent_decode_str;

use yew::prelude::*;
use yew_router::{HashRouter, Routable, Switch};
use yew_router::scope_ext::RouterScopeExt;

use pwt::prelude::*;
use pwt::touch::{Fab, FabMenu, FabMenuAlign};
use pwt::widget::{Column, Container, Dialog, ThemeLoader};

use proxmox_yew_comp::http_login;
use proxmox_yew_comp::{LoginInfo, LoginPanel, ProxmoxProduct};

use crate::{Route, SpamList, TopNavBar};

pub struct PageSpamList;

pub enum Msg {
    Preview(String),
}

impl Component for PageSpamList {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
           Msg::Preview(id) => {
                //log::info!("Preview {id}");
                let navigator = ctx.link().navigator().unwrap();
                navigator.push(&Route::ViewMail { id: id.clone() });
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = SpamList::new()
            .on_preview(ctx.link().callback(|id| Msg::Preview(id)));

        let fab = Container::new()
            .class("pwt-position-fixed")
            .class("pwt-right-2 pwt-bottom-4")
            .with_child(
                FabMenu::new()
                    .align(FabMenuAlign::End)
                    .main_icon_class("fa fa-calendar")
                    .main_button_class("pwt-scheme-primary")
            );

        Column::new()
            .class("pwt-viewport")
            .with_child(TopNavBar::new())
            .with_child(content)
            .with_child(fab)
            .into()
    }
}
