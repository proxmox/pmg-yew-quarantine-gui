use anyhow::Error;
use wasm_bindgen::JsValue;

use core::clone::Clone;
use js_sys::Date;
use serde_json::Value;
use std::rc::Rc;

use pwt::{
    css::{AlignItems, ColorScheme, FlexFit, Opacity, Overflow},
    prelude::*,
    touch::{Slidable, SlidableAction, SnackBar, SnackBarContextExt},
    widget::{error_message, Container, Fa, List, ListTile, Progress, Row},
};
use yew::{
    html::{IntoEventCallback, IntoPropValue},
    virtual_dom::{VComp, VNode},
};
//use yew::html::IntoEventCallback;

use proxmox_yew_comp::http_get;
use pwt::widget::Column;

use serde::{Deserialize, Serialize};

use crate::{mail_action, MailAction};

#[derive(Copy, Clone, Serialize, Default, PartialEq)]
pub struct SpamListParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    starttime: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    endtime: Option<u64>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MailInfo {
    //pub bytes: i64,
    pub from: String,
    pub id: String,
    pub subject: String,
    //pub receiver: String,
    //pub envelope_sender: String,
    pub spamlevel: i64,
    pub time: i64,
}

#[derive(Clone)]
pub enum ListEntry {
    Date(String),
    Mail(MailInfo),
}

#[derive(Clone, PartialEq, Properties)]
pub struct SpamList {
    #[prop_or_default]
    on_preview: Option<Callback<String>>,
    #[prop_or_default]
    param: SpamListParam,
}

impl SpamList {
    pub fn new() -> Self {
        yew::props!(Self {})
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
    Action(String, MailAction), // id
    LoadResult(Result<Vec<MailInfo>, Error>),
}

pub struct PmgSpamList {
    data: Option<Result<Vec<ListEntry>, Error>>,
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
}

impl Component for PmgSpamList {
    type Message = Msg;
    type Properties = SpamList;

    fn create(ctx: &Context<Self>) -> Self {
        let me = Self { data: None };
        me.load(ctx);
        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Reload => {
                self.load(ctx);
            }
            Msg::LoadResult(result) => match result {
                Ok(mut data) => {
                    data.sort_by(|a, b| match b.time.cmp(&a.time) {
                        core::cmp::Ordering::Equal => a.cmp(b),
                        other => other,
                    });

                    let mut res = Vec::new();
                    let mut last_date = String::new();

                    for mail in data {
                        let date = epoch_to_date(mail.time);
                        if date != last_date {
                            res.push(ListEntry::Date(date.clone()));
                            last_date = date;
                        }
                        res.push(ListEntry::Mail(mail));
                    }

                    self.data = Some(Ok(res));
                }
                Err(err) => self.data = Some(Err(err)),
            },
            Msg::Action(id, action) => {
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(err) = mail_action(&id, action).await {
                        link.show_snackbar(SnackBar::new().message(err.to_string()));
                    }
                    link.send_message(Msg::Reload);
                });
                return false;
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
            Some(Ok(data)) if !data.is_empty() => {
                let on_preview = ctx.props().on_preview.clone();
                let data = data.clone();
                let link = ctx.link().clone();
                List::new(data.len() as u64, move |pos: u64| {
                    ListTile::new().padding(0).with_child(render_list_item(
                        &link,
                        on_preview.clone(),
                        &data[pos as usize],
                    ))
                })
                .class(FlexFit)
                .into()
            }
            Some(Ok(_)) => Row::new()
                .padding(2)
                .with_child(tr!("No data in database"))
                .into(),
            Some(Err(err)) => error_message(&err.to_string()).into(),
            None => Progress::new().into(),
        }
    }
}

impl From<SpamList> for VNode {
    fn from(val: SpamList) -> Self {
        let comp = VComp::new::<PmgSpamList>(Rc::new(val), None);
        VNode::from(comp)
    }
}

fn render_list_item(
    link: &yew::html::Scope<PmgSpamList>,
    on_preview: Option<Callback<String>>,
    item: &ListEntry,
) -> Html {
    match item {
        ListEntry::Date(date) => Container::new()
            .padding(1)
            .class(ColorScheme::Surface)
            .class("pwt-default-colors")
            .with_child(date)
            .into(),
        ListEntry::Mail(item) => {
            let make_cb = |action: MailAction| {
                let id = item.id.clone();
                link.callback(move |_| Msg::Action(id.clone(), action))
            };

            let content = Column::new()
                .class(FlexFit)
                .with_child(html! {
                    <div class="pwt-font-label-small pwt-text-truncate">{&item.from}</div>
                })
                .with_child(html! {
                    <div class="pwt-font-title-small pwt-text-truncate">{&item.subject}</div>
                });
            let score = Container::new()
                .class("pwt-white-space-nowrap")
                .class(Opacity::Half)
                .with_child(tr!("Score: {0}", item.spamlevel));
            let main = Row::new()
                .class(FlexFit)
                .gap(1)
                .padding_x(2)
                .padding_y(1)
                .border_bottom(true)
                .class(AlignItems::Center)
                .style("cursor", "pointer")
                .with_child(content)
                .with_child(score)
                .with_child(Fa::new("chevron-right").class(Opacity::Half));

            Slidable::new(main)
                .class(Overflow::Auto)
                .on_tap({
                    let id = item.id.clone();
                    let on_preview = on_preview.clone();
                    move |_| {
                        if let Some(on_preview) = &on_preview {
                            on_preview.emit(id.clone())
                        }
                    }
                })
                .left_actions(
                    Row::new()
                        .style("height", "100%") // FIXME better solved in scss of slidable?
                        .class(AlignItems::Center)
                        .with_child(
                            SlidableAction::new(tr!("Deliver"))
                                .class(ColorScheme::SuccessContainer)
                                .icon_class("fa fa-paper-plane")
                                .on_activate(make_cb(MailAction::Deliver)),
                        )
                        .with_child(
                            SlidableAction::new(tr!("Welcomelist"))
                                .icon_class("fa fa-check")
                                .on_activate(make_cb(MailAction::Welcomelist)),
                        ),
                )
                .right_actions(
                    Row::new()
                        .style("height", "100%") // FIXME better solved in scss of slidable?
                        .class(AlignItems::Center)
                        .with_child(
                            SlidableAction::new(tr!("Blocklist"))
                                .class(ColorScheme::WarningContainer)
                                .icon_class("fa fa-times")
                                .on_activate(make_cb(MailAction::Blocklist)),
                        )
                        .with_child(
                            SlidableAction::new(tr!("Delete"))
                                .class(ColorScheme::ErrorContainer)
                                .icon_class("fa fa-trash")
                                .on_activate(make_cb(MailAction::Delete)),
                        ),
                )
                .into()
        }
    }
}

fn epoch_to_date(epoch: i64) -> String {
    let date = Date::new(&JsValue::from_f64(1000.0 * epoch as f64));
    format!(
        "{:04}-{:02}-{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date()
    )
}
