use std::rc::Rc;

use serde_json::{json, Value};
use anyhow::Error;

use pwt::{prelude::*, widget::AlertDialog};
use yew::{virtual_dom::{VComp, VNode}, html::IntoEventCallback};
//use yew::html::IntoEventCallback;
use yew_router::scope_ext::RouterScopeExt;

use proxmox_yew_comp::http_post;
use pwt::widget::{ActionIcon, Button, Container, Column, Row};
use pwt::touch::{Fab, FabMenu, FabMenuAlign};

use super::{record_data_change, Route};

#[derive(Clone, PartialEq, Properties)]
pub struct PageMailView {
    id: String,
}

impl PageMailView {
    pub fn new(id: impl Into<String>) -> Self {
        yew::props!(Self { id: id.into() })
    }
}

pub enum Msg {
    ClearError,
    ActionResult(Result<Value, Error>),
}
pub struct PmgPageMailView {
    error: Option<String>,
}

impl PmgPageMailView {

    fn action_callback(&self, ctx: &Context<Self>, action: &str) -> Callback<MouseEvent> {
        let props = ctx.props();

        let link = ctx.link().clone();
        let param = json!({
            "action": action,
            "id": props.id,
        });

        Callback::from(move |_event: MouseEvent| {
            let param = param.clone();
            let link = link.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = http_post("/quarantine/content", Some(param)).await;
                link.send_message(Msg::ActionResult(result));
            });
        })
    }

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

impl Component for PmgPageMailView {
    type Message = Msg;
    type Properties = PageMailView;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { error: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ClearError => {
                self.error = None;
            }
            Msg::ActionResult(result) => {
                //log::info!("RESULT {:?}", result);
                record_data_change();

                if let Err(err) = result {
                    self.error = Some(err.to_string());
                    //log::info!("ERROR {:?}", self.error);
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let blacklist_button = Fab::new("fa fa-times")
            .text("Blacklist")
            .on_click(self.action_callback(ctx, "blacklist"));

        let whitelist_button = Fab::new("fa fa-check")
            .text("Whitelist")
            .on_click(self.action_callback(ctx, "whitelist"));

        let delete_button = Fab::new("fa fa-trash")
            .text("Delete")
            .on_click(self.action_callback(ctx, "delete"));

        let deliver_button = Fab::new("fa fa-paper-plane")
            .text("Deliver")
            .on_click(self.action_callback(ctx, "deliver"));

        let fab = Container::new()
            .class("pwt-position-fixed")
            .class("pwt-right-2 pwt-bottom-4")
            .with_child(
                FabMenu::new()
                    .align(FabMenuAlign::End)
                    .main_button_class("pwt-scheme-primary")
                    .with_child(blacklist_button)
                    .with_child(whitelist_button)
                    .with_child(delete_button)
                    .with_child(deliver_button)
            );

        let error_dialog = match &self.error {
            Some(msg) => {
                Some(
                    AlertDialog::new(msg)
                        .on_close(ctx.link().callback(|_| Msg::ClearError))
                )
            }
            None => None,
        };

        Column::new()
            .class("pwt-viewport")
            .with_child(self.top_bar(ctx))
            .with_child(self.content_view(ctx))
            .with_optional_child(error_dialog)
            .with_child(fab)
            .into()
    }
}

impl Into<VNode> for PageMailView {
    fn into(self) -> VNode {
        let comp = VComp::new::<PmgPageMailView>(Rc::new(self), None);
        VNode::from(comp)
    }
}
