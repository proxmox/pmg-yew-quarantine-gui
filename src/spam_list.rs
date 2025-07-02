use anyhow::{format_err, Error};

use serde_json::Value;
use std::rc::Rc;

use pwt::{prelude::*, widget::error_message};
use yew::{
    html::{IntoEventCallback, IntoPropValue},
    virtual_dom::{VComp, VNode},
};
//use yew::html::IntoEventCallback;

use proxmox_yew_comp::http_get;
use pwt::state::SharedStateObserver;
use pwt::widget::{Card, Column};

use serde::{Deserialize, Serialize};

use crate::ReloadController;

#[derive(Copy, Clone, Serialize, Default, PartialEq)]
pub struct SpamListParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    starttime: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    endtime: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct MailInfo {
    pub bytes: i64,
    pub from: String,
    pub id: String,
    pub subject: String,
    pub receiver: String,
    pub envelope_sender: String,
    pub spamlevel: i64,
    pub time: u64,
}
#[derive(Clone, PartialEq, Properties)]
pub struct SpamList {
    #[prop_or_default]
    on_preview: Option<Callback<String>>,
    #[prop_or_default]
    param: SpamListParam,

    reload_controller: ReloadController,
}

impl SpamList {
    pub fn new(reload_controller: ReloadController) -> Self {
        yew::props!(Self { reload_controller })
    }

    pub fn starttime(mut self, epoch: impl IntoPropValue<Option<u64>>) -> Self {
        self.param.starttime = epoch.into_prop_value();
        self
    }

    pub fn endtime(mut self, epoch: impl IntoPropValue<Option<u64>>) -> Self {
        self.param.endtime = epoch.into_prop_value();
        self
    }

    pub fn on_preview(mut self, cb: impl IntoEventCallback<String>) -> Self {
        self.on_preview = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    Reload,
    LoadResult(Result<Vec<MailInfo>, Error>),
}

pub struct PmgSpamList {
    data: Result<Vec<MailInfo>, Error>,
    reload_observer: SharedStateObserver<usize>,
}

impl PmgSpamList {
    fn load(&self, ctx: &Context<Self>) {
        let props = ctx.props();
        let link = ctx.link().clone();
        let param: Value = serde_json::to_value(props.param).unwrap();

        wasm_bindgen_futures::spawn_local(async move {
            let result = http_get::<Vec<MailInfo>>("/quarantine/spam", Some(param)).await;
            link.send_message(Msg::LoadResult(result));
        })
    }

    fn render_list_item(&self, ctx: &Context<Self>, item: &MailInfo) -> Html {
        Card::new()
            .class("pwt-d-flex")
            .class("pwt-shape-none pwt-card-flat pwt-interactive")
            .class("pwt-scheme-neutral")
            .padding_x(2)
            .padding_y(1)
            .border_bottom(true)
            .class("pwt-align-items-center")
            .with_child(
                Column::new()
                    .class("pwt-fit")
                    .class("pwt-pe-1")
                    .with_child(html! {
                        <div class="pwt-font-label-small pwt-text-truncate">{&item.from}</div>
                    })
                    .with_child(html! {
                        <div class="pwt-font-title-medium pwt-text-truncate">{&item.subject}</div>
                    }),
            )
            .with_child(html! {
                <div class="pwt-white-space-nowrap">
                {format!("Score: {}", item.spamlevel)}
                </div>
            })
            .onclick(Callback::from({
                let id = item.id.clone();
                let on_preview = ctx.props().on_preview.clone();
                move |_| {
                    if let Some(on_preview) = &on_preview {
                        on_preview.emit(id.clone());
                    }
                }
            }))
            .into()
    }
}

impl Component for PmgSpamList {
    type Message = Msg;
    type Properties = SpamList;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let reload_observer = props
            .reload_controller
            .add_listener(ctx.link().callback(|_| Msg::Reload));

        let me = Self {
            data: Err(format_err!("no data loaded")),
            reload_observer,
        };
        me.load(ctx);
        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Reload => {
                self.load(ctx);
            }
            Msg::LoadResult(result) => {
                self.data = result;
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if props.reload_controller != old_props.reload_controller {
            self.reload_observer = props
                .reload_controller
                .add_listener(ctx.link().callback(|_| Msg::Reload));
        }

        if props.param != old_props.param {
            self.load(ctx);
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.data {
            Ok(data) => {
                let children: Vec<Html> = data
                    .iter()
                    .map(|item| self.render_list_item(ctx, item))
                    .collect();

                Column::new().class("pwt-fit").children(children).into()
            }
            Err(err) => error_message(&err.to_string()).into(),
        }
    }
}

impl Into<VNode> for SpamList {
    fn into(self) -> VNode {
        let comp = VComp::new::<PmgSpamList>(Rc::new(self), None);
        VNode::from(comp)
    }
}
