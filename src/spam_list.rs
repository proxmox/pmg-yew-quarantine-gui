use std::{rc::Rc, str::FromStr};

use anyhow::{format_err, Error};
use gloo_utils::window;
use js_sys::Date;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;
use wasm_bindgen::JsValue;
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::{VComp, VNode};

use pwt::{
    css::{AlignItems, ColorScheme, FlexFit, Opacity, Overflow},
    prelude::*,
    state::SharedStateObserver,
    touch::{Slidable, SlidableAction, SnackBar, SnackBarContextExt},
    widget::{error_message, Container, Fa, List, ListTile, Progress, Row},
};

use proxmox_yew_comp::http_get;
use pwt::widget::Column;

use crate::{mail_action, MailAction, QuarantineReload};

#[derive(Copy, Clone, Serialize, Default, PartialEq)]
pub struct SpamListParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    starttime: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    endtime: Option<u64>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MailInfo {
    //pub bytes: i64,
    pub from: String,
    pub id: String,
    pub subject: String,
    //pub receiver: String,
    //pub envelope_sender: String,
    pub spamlevel: i64,
    // sum of the positive resp. negative spam test scores; these are fractional,
    // hence f64 (which is why MailInfo cannot derive Eq/Ord)
    #[serde(rename = "score-positive", default)]
    pub score_positive: f64,
    #[serde(rename = "score-negative", default)]
    pub score_negative: f64,
    // whether the mail was marked as seen; accept both JSON bool and the 1/0 a
    // Perl API may emit
    #[serde(default, deserialize_with = "deserialize_flexible_bool")]
    pub seen: bool,
    pub time: i64,
}

fn deserialize_flexible_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::Bool(b) => b,
        Value::Number(n) => n.as_i64().is_some_and(|v| v != 0),
        Value::String(s) => s == "1" || s.eq_ignore_ascii_case("true"),
        _ => false,
    })
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
    // kept alive to keep the reload listener registered on the shared trigger
    _reload_observer: Option<SharedStateObserver<usize>>,
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
        // The page stack keeps this list mounted while the mail view sits on top,
        // so it would otherwise not notice actions taken there. Reload on every
        // bump of the shared trigger to stay in sync once the user returns.
        let reload_observer = ctx
            .link()
            .context::<QuarantineReload>(Callback::noop())
            .map(|(reload, _handle)| reload.0.add_listener(ctx.link().callback(|_| Msg::Reload)));

        let me = Self {
            data: None,
            _reload_observer: reload_observer,
        };

        match extract_mail_action_from_query_params() {
            Ok(None) => {}
            Ok(Some((id, action))) => {
                ctx.link().send_message(Msg::Action(id, action));
            }
            Err(err) => {
                ctx.link().show_snackbar(
                    SnackBar::new().message(format!("could not execute action: {err}")),
                );
            }
        }

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
                    data.sort_by(|a, b| b.time.cmp(&a.time).then_with(|| a.id.cmp(&b.id)));

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
                    let msg = match mail_action(&id, action).await {
                        Ok(_) => tr!("Action '{0}' successful", action),
                        Err(err) => err.to_string(),
                    };
                    link.show_snackbar(SnackBar::new().message(msg));
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
            // show the net score plus the separate sums of positive and negative
            // test scores, which gives a better feel for borderline mails
            let score = Column::new()
                .class("pwt-white-space-nowrap")
                .class(Opacity::Half)
                .with_child(tr!("Score: {0}", item.spamlevel))
                .with_child(html! {
                    <span class="pwt-font-label-small">
                        { format!("+{:.1} / {:.1}", item.score_positive, item.score_negative) }
                    </span>
                });
            let mut main = Row::new()
                .class(FlexFit)
                .gap(1)
                .padding_x(2)
                .padding_y(1)
                .border_bottom(true)
                .class(AlignItems::Center)
                .style("cursor", "pointer");
            // dim seen mails and flag them with a leading marker
            if item.seen {
                main = main.class(Opacity::Half).with_child(Fa::new("check"));
            }
            let main = main
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
                        )
                        .with_child(
                            SlidableAction::new(tr!("Mark as Seen"))
                                .icon_class("fa fa-eye")
                                .on_activate(make_cb(MailAction::MarkSeen)),
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

fn extract_mail_action_from_query_params() -> Result<Option<(String, MailAction)>, Error> {
    let id = extract_query_parameter("cselect")?;
    let action = extract_query_parameter("action")?;

    if let (Some(id), Some(action)) = (id, action) {
        let action = MailAction::from_str(&action)?;
        return Ok(Some((id, action)));
    }
    Ok(None)
}

/// Removes `name` parameter from the get values via the browser `history` object and returns it
/// if it exists.
pub fn extract_query_parameter(name: &str) -> Result<Option<String>, Error> {
    let location = window().location();
    let history = window().history().unwrap();
    let search = location.search().unwrap();
    let param = web_sys::UrlSearchParams::new_with_str(&search).unwrap();

    if let Some(value) = param.get(name) {
        param.delete(name);

        let mut url = Url::parse(
            &location
                .href()
                .map_err(|err| format_err!("could not get location: {err:?}"))?,
        )?;
        let query: String = param.to_string().into();

        url.set_query(Some(&query));
        history
            .replace_state_with_url(&JsValue::null(), "", Some(url.as_str()))
            .map_err(|err| format_err!("could not set url: {err:?}"))?;

        return Ok(Some(value));
    }

    Ok(None)
}
