use anyhow::{format_err, Error};

use proxmox_schema::param_format_err;
use serde_json::{json, Value};
use std::rc::Rc;

use pwt::{prelude::*, widget::error_message};
use yew::{
    html::{IntoEventCallback, IntoPropValue},
    virtual_dom::{VComp, VNode},
};
//use yew::html::IntoEventCallback;

use proxmox_yew_comp::http_get;
use pwt::widget::{Column, Row};

use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Serialize, Default, PartialEq)]
struct SpamListParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    starttime: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    endtime: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct MailInfo {
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
    on_preview: Option<Callback<String>>,
    #[prop_or_default]
    param: SpamListParam,
}

impl SpamList {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn starttime(mut self, epoch: impl IntoPropValue<Option<u64>>) -> Self  {
        self.param.starttime = epoch.into_prop_value();
        self
    }

    pub fn endtime(mut self, epoch: impl IntoPropValue<Option<u64>>) -> Self  {
        self.param.endtime = epoch.into_prop_value();
        self
    }

    pub fn on_preview(mut self, cb: impl IntoEventCallback<String>) -> Self {
        self.on_preview = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    LoadResult(Result<Vec<MailInfo>, Error>),
}

pub struct PmgSpamList {
    data: Result<Vec<MailInfo>, Error>,
}

impl PmgSpamList {
    fn load(&self, ctx: &Context<Self>) {
        let props = ctx.props();
        let link = ctx.link().clone();
        let mut param: Value = serde_json::to_value(props.param).unwrap();

        wasm_bindgen_futures::spawn_local(async move {

            let result = http_get::<Vec<MailInfo>>("/quarantine/spam", Some(param)).await;
            link.send_message(Msg::LoadResult(result));
        })
    }

    fn render_list_item(&self, ctx: &Context<Self>, item: &MailInfo) -> Html {
        Row::new()
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
        let me = Self {
            data: Err(format_err!("no data loaded")),
        };
        me.load(ctx);
        me
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadResult(result) => {
                self.data = result;
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

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
            Err(err) => error_message(&err.to_string(), "pwt-p-2"),
        }
    }
}

impl Into<VNode> for SpamList {
    fn into(self) -> VNode {
        let comp = VComp::new::<PmgSpamList>(Rc::new(self), None);
        VNode::from(comp)
    }
}
