use serde_json::Value;
use std::rc::Rc;

use pwt::prelude::*;
use yew::virtual_dom::{VComp, VNode};
//use yew::html::IntoEventCallback;

use proxmox_yew_comp::http_get;
use pwt::widget::{Column, Row};

use serde::Deserialize;

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
pub struct SpamList {}

impl SpamList {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub enum Msg {
    LoadData(Vec<MailInfo>),
}

pub struct PmgSpamList {
    data: Vec<MailInfo>,
}

impl PmgSpamList {
    fn load(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            match http_get::<Vec<MailInfo>>("/quarantine/spam", None).await {
                Ok(data) => {
                    link.send_message(Msg::LoadData(data));
                }
                Err(err) => {
                    log::error!("ERROR: {:?}", err);
                    //link.send_message(Msg::LoginError(err.to_string()));
                }
            }
        })
    }
}

impl Component for PmgSpamList {
    type Message = Msg;
    type Properties = SpamList;

    fn create(ctx: &Context<Self>) -> Self {
        let me = Self { data: Vec::new() };
        me.load(ctx);
        me
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadData(data) => {
                log::info!("GOT {:?}", data);
                self.data = data;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let children: Vec<Html> = self.data.iter()
            .map(|item| {
                Row::new()
                    .padding_x(2)
                    .padding_y(1)
                    .border_bottom(true)
                    .class("pwt-align-items-center")
                    .with_child(
                        Column::new()
                            .class("pwt-fit")
                            .class("pwt-pe-1")
                            .with_child(html!{
                                <div class="pwt-font-label-small pwt-text-truncate">{&item.from}</div>
                            })
                            .with_child(html!{
                                <div class="pwt-font-title-medium pwt-text-truncate">{&item.subject}</div>
                            })
                    )
                    .with_child(html!{
                        <div class="pwt-white-space-nowrap">
                        {format!("Score: {}", item.spamlevel)}
                        </div>
                    })
                    .into()
            })
            .collect();

        Column::new()
            .class("pwt-fit")
            .children(children)
            .into()
    }
}

impl Into<VNode> for SpamList {
    fn into(self) -> VNode {
        let comp = VComp::new::<PmgSpamList>(Rc::new(self), None);
        VNode::from(comp)
    }
}
