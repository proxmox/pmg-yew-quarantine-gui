use serde_json::Value;
use std::rc::Rc;

use pwt::prelude::*;
use yew::{virtual_dom::{VComp, VNode}, html::IntoEventCallback};
//use yew::html::IntoEventCallback;
use yew_router::scope_ext::RouterScopeExt;

use proxmox_yew_comp::http_get;
use pwt::widget::{ActionIcon, Button, Container, Column, Row};
use pwt::touch::{Fab, FabMenu, FabMenuAlign};

use super::Route;

#[derive(Clone, PartialEq, Properties)]
pub struct MailView {
    id: String,
}

impl MailView {
    pub fn new(id: impl Into<String>) -> Self {
        yew::props!(Self { id: id.into() })
    }
}

pub struct PmgMailView {
}

impl PmgMailView {

    fn top_bar(&self, ctx: &Context<Self>) -> Html {
        Row::new()
        .attribute("role", "banner")
        .attribute("aria-label", "Spam Mail Preview")
        .class("pwt-navbar")
        .class("pwt-justify-content-space-between pwt-align-items-center")
        .class("pwt-border-bottom")
        .class("pwt-shadow1")
        .padding(1)
        .with_child(
            Row::new()
                .class("pwt-align-items-center")
                .with_child(
                    ActionIcon::new("fa fa-arrow-left")
                        .class("pwt-font-size-headline-small")
                        .class("pwt-color-primary")
                        .on_activate({
                            let link = ctx.link().clone();
                            move |_| {
                                let navigator = link.navigator().unwrap();
                                navigator.push(&Route::SpamList);
                            }
                        })
                )
                .with_child(html!{
                    <span class="pwt-ps-1 pwt-font-headline-medium pwt-user-select-none">{"Preview"}</span>
                })

        )
        //    .with_flex_spacer()
        //        .with_child(button_group)
        .into()
    }

    fn content_view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let iframe = html!{
            <iframe frameborder="0" width="100%" height="100%" sandbox="allow-same-origin"
                src={format!("/api2/htmlmail/quarantine/content?id={}", props.id)}>
            </iframe>
        };

        Container::new()
            .class("pwt-flex-fill")
            .with_child(iframe)
            .into()
    }
}

impl Component for PmgMailView {
    type Message = ();
    type Properties = MailView;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let fab = Container::new()
            .class("pwt-position-fixed")
            .class("pwt-right-2 pwt-bottom-4")
            .with_child(
                FabMenu::new()
                    .align(FabMenuAlign::End)
                    .main_button_class("pwt-scheme-primary")
                    .with_child(Fab::new("fa fa-times").text("Blacklist"))
                    .with_child(Fab::new("fa fa-check").text("Whitelist"))
                    .with_child(Fab::new("fa fa-trash").text("Delete"))
                    .with_child(Fab::new("fa fa-paper-plane").text("Deliver")),
            );

        Column::new()
            .class("pwt-viewport")
            .with_child(self.top_bar(ctx))
            .with_child(self.content_view(ctx))
            .with_child(fab)
            .into()
    }
}

impl Into<VNode> for MailView {
    fn into(self) -> VNode {
        let comp = VComp::new::<PmgMailView>(Rc::new(self), None);
        VNode::from(comp)
    }
}
