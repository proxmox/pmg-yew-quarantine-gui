use std::rc::Rc;

use anyhow::Error;
use serde_json::Value;

use pwt::{prelude::*, widget::AlertDialog};
use yew::virtual_dom::{VComp, VNode};
//use yew::html::IntoEventCallback;

use pwt::css::FlexFit;
use pwt::touch::{ApplicationBar, FabMenu, FabMenuEntry, Scaffold};
use pwt::widget::Container;

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
    ClearError,
    ActionResult(Result<Value, Error>),
}
pub struct PmgPageMailView {
    error: Option<String>,
}

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
        Self { error: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ClearError => {
                self.error = None;
            }
            Msg::ActionResult(result) => {
                if let Err(err) = result {
                    self.error = Some(err.to_string());
                    //log::info!("ERROR {:?}", self.error);
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
                tr!("Whitelist"),
                "fa fa-check",
                self.action_callback(ctx, MailAction::Whitelist),
            ))
            .with_child(FabMenuEntry::new(
                tr!("Blacklist"),
                "fa fa-times",
                self.action_callback(ctx, MailAction::Blacklist),
            ));

        let error_dialog = match &self.error {
            Some(msg) => {
                Some(AlertDialog::new(msg).on_close(ctx.link().callback(|_| Msg::ClearError)))
            }
            None => None,
        };

        Scaffold::new()
            .application_bar(ApplicationBar::new().title(tr!("Preview")))
            .body(
                Container::new()
                    .class(FlexFit)
                    .with_child(self.content_view(ctx))
                    .with_optional_child(error_dialog),
            )
            .favorite_action_button(fab)
            .into()
    }
}

impl Into<VNode> for PageMailView {
    fn into(self) -> VNode {
        let comp = VComp::new::<PmgPageMailView>(Rc::new(self), None);
        VNode::from(comp)
    }
}
