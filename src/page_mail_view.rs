use std::rc::Rc;

use anyhow::Error;
use serde_json::Value;

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::touch::{ApplicationBar, FabMenu, FabMenuEntry, Scaffold, SnackBar, SnackBarContextExt};

use crate::{mail_action, MailAction};

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
    ActionResult(Result<Value, Error>),
}
pub struct PmgPageMailView {}

impl PmgPageMailView {
    fn action_callback(&self, ctx: &Context<Self>, action: MailAction) -> Callback<MouseEvent> {
        let props = ctx.props();

        let link = ctx.link().clone();
        let id = props.id.clone();

        Callback::from(move |_event: MouseEvent| {
            let link = link.clone();
            let id = id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = mail_action(&id, action).await;
                link.send_message(Msg::ActionResult(result));
            });
        })
    }

    fn content_view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        html! {
            <iframe frameborder="0" width="100%" height="100%" sandbox="allow-same-origin"
                src={format!("/api2/htmlmail/quarantine/content?id={}", props.id)}>
            </iframe>
        }
    }
}

impl Component for PmgPageMailView {
    type Message = Msg;
    type Properties = PageMailView;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ActionResult(result) => {
                if let Err(err) = result {
                    ctx.link()
                        .show_snackbar(SnackBar::new().message(err.to_string()));
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let fab = FabMenu::new()
            .main_icon_class("fa fa-bars")
            .with_child(FabMenuEntry::new(
                tr!("Deliver"),
                "fa fa-paper-plane",
                self.action_callback(ctx, MailAction::Deliver),
            ))
            .with_child(FabMenuEntry::new(
                tr!("Delete"),
                "fa fa-trash",
                self.action_callback(ctx, MailAction::Delete),
            ))
            .with_child(FabMenuEntry::new(
                tr!("Welcomelist"),
                "fa fa-check",
                self.action_callback(ctx, MailAction::Welcomelist),
            ))
            .with_child(FabMenuEntry::new(
                tr!("Blocklist"),
                "fa fa-times",
                self.action_callback(ctx, MailAction::Blocklist),
            ));

        Scaffold::new()
            .application_bar(ApplicationBar::new().title(tr!("Preview")))
            .body(self.content_view(ctx))
            .favorite_action_button(fab)
            .into()
    }
}

impl From<PageMailView> for VNode {
    fn from(val: PageMailView) -> Self {
        let comp = VComp::new::<PmgPageMailView>(Rc::new(val), None);
        VNode::from(comp)
    }
}
