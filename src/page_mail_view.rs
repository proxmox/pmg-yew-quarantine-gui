use std::rc::Rc;

use anyhow::Error;
use serde_json::{json, Value};

use pwt::{prelude::*, widget::AlertDialog};
use yew::virtual_dom::{VComp, VNode};
//use yew::html::IntoEventCallback;

use pwt::css::FlexFit;
use pwt::touch::{ApplicationBar, FabMenu, FabMenuAlign, FabMenuEntry, Scaffold};
use pwt::widget::Container;

use proxmox_yew_comp::http_post;

use super::ReloadController;

#[derive(Clone, PartialEq, Properties)]
pub struct PageMailView {
    id: String,
    reload_controller: ReloadController,
}

impl PageMailView {
    pub fn new(reload_controller: ReloadController, id: impl Into<String>) -> Self {
        yew::props!(Self {
            id: id.into(),
            reload_controller,
        })
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

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ClearError => {
                self.error = None;
            }
            Msg::ActionResult(result) => {
                //log::info!("RESULT {:?}", result);
                props.reload_controller.reload();

                if let Err(err) = result {
                    self.error = Some(err.to_string());
                    //log::info!("ERROR {:?}", self.error);
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let blacklist_button = FabMenuEntry::new(
            tr!("Blacklist"),
            "fa fa-times",
            self.action_callback(ctx, "blacklist"),
        );

        let whitelist_button = FabMenuEntry::new(
            tr!("Whitelist"),
            "fa fa-check",
            self.action_callback(ctx, "whitelist"),
        );

        let delete_button = FabMenuEntry::new(
            tr!("Delete"),
            "fa fa-trash",
            self.action_callback(ctx, "delete"),
        );

        let deliver_button = FabMenuEntry::new(
            tr!("Deliver"),
            "fa fa-paper-plane",
            self.action_callback(ctx, "deliver"),
        );

        let fab = FabMenu::new()
            .align(FabMenuAlign::End)
            .main_button_class("pwt-scheme-primary")
            .with_child(blacklist_button)
            .with_child(whitelist_button)
            .with_child(delete_button)
            .with_child(deliver_button);

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
