use std::rc::Rc;

use anyhow::Error;
use js_sys::Date;
use wasm_bindgen::JsValue;

use yew::platform::spawn_local;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew_router::scope_ext::RouterScopeExt;

use pwt::css::{ColorScheme, FlexFit, JustifyContent};
use pwt::prelude::*;
use pwt::touch::{ApplicationBar, Fab, Scaffold};
use pwt::widget::form::{Field, Form, FormContext, InputType};
use pwt::widget::{Button, Column, Dialog, Image, Row, ThemeModeSelector};

use proxmox_subscription::{SubscriptionInfo, SubscriptionStatus};
use proxmox_yew_comp::http_get;

use crate::{Route, SpamList};

#[derive(Clone, PartialEq, Properties)]
pub struct PageSpamList {}

impl PageSpamList {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for PageSpamList {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ViewState {
    Normal,
    ShowDialog,
    ShowSubscriptionNotice,
}
pub struct PmgPageSpamList {
    state: ViewState,
    start_date: f64,
    end_date: f64,
    form_context: FormContext,
    subscription_result: Option<bool>,
}

pub enum Msg {
    Preview(String),
    ShowDialog,
    ShowSubscriptionNotice,
    CloseDialog,
    ApplyDate,
    SubscriptionResult(Result<SubscriptionInfo, Error>),
}

fn epoch_to_date_string(epoch: f64) -> String {
    let start_date = Date::new(&JsValue::from_f64(epoch));
    format!(
        "{:04}-{:02}-{:02}",
        start_date.get_full_year(),
        start_date.get_month() + 1,
        start_date.get_date(),
    )
}
impl PmgPageSpamList {
    fn date_range_form(&self, ctx: &Context<Self>) -> Html {
        let start_date = epoch_to_date_string(self.start_date);
        let end_date = epoch_to_date_string(self.end_date);

        let panel = Column::new()
            .padding(2)
            .gap(1)
            .min_width("70vw")
            .class("pwt-flex-fill")
            .with_child(tr!("From:"))
            .with_child(
                Field::new()
                    .name("from")
                    .default(start_date)
                    .input_type(InputType::Date),
            )
            .with_child(tr!("To:"))
            .with_child(
                Field::new()
                    .name("to")
                    .default(end_date)
                    .input_type(InputType::Date),
            )
            .with_child(
                Row::new().class("pwt-pt-2").with_flex_spacer().with_child(
                    Button::new(tr!("Apply"))
                        .class("pwt-scheme-primary")
                        .onclick(ctx.link().callback(|_| Msg::ApplyDate)),
                ),
            );

        Form::new()
            .form_context(self.form_context.clone())
            .with_child(panel)
            .into()
    }
}

impl Component for PmgPageSpamList {
    type Message = Msg;
    type Properties = PageSpamList;

    fn create(ctx: &Context<Self>) -> Self {
        let start_date = js_sys::Date::new_0();
        start_date.set_hours(0);
        start_date.set_minutes(0);
        start_date.set_seconds(0);
        start_date.set_milliseconds(0);

        let mut start_date = start_date.get_time();
        let end_date = start_date + 24.0 * 3600000.0;
        start_date = end_date - 7.0 * 24.0 * 3600000.0;

        let link = ctx.link().clone();
        spawn_local(async move {
            let result = http_get("/nodes/localhost/subscription", None).await;
            link.send_message(Msg::SubscriptionResult(result));
        });

        Self {
            state: ViewState::Normal,
            start_date,
            end_date,
            form_context: FormContext::new(),
            subscription_result: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ShowDialog => {
                self.state = ViewState::ShowDialog;
                true
            }
            Msg::ShowSubscriptionNotice => {
                self.state = ViewState::ShowSubscriptionNotice;
                true
            }
            Msg::CloseDialog => {
                self.state = ViewState::Normal;
                true
            }
            Msg::ApplyDate => {
                self.state = ViewState::Normal;

                let start = self.form_context.read().get_field_value("from").unwrap();
                self.start_date = Date::parse(start.as_str().unwrap());
                let end = self.form_context.read().get_field_value("to").unwrap();
                self.end_date = Date::parse(end.as_str().unwrap());

                true
            }
            Msg::Preview(id) => {
                //log::info!("Preview {id}");
                let navigator = ctx.link().navigator().unwrap();
                navigator.push(&Route::ViewMail { id: id.clone() });
                true
            }
            Msg::SubscriptionResult(result) => {
                let valid = match result {
                    Ok(subscription) => matches!(subscription.status, SubscriptionStatus::Active),
                    Err(_) => false,
                };
                if !valid {
                    self.state = ViewState::ShowSubscriptionNotice;
                }
                self.subscription_result = Some(valid);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = SpamList::new()
            .starttime((self.start_date / 1000.0) as u64)
            .endtime((self.end_date / 1000.0) as u64)
            .on_preview(ctx.link().callback(Msg::Preview));

        let dialog = match self.state {
            ViewState::Normal => None,
            ViewState::ShowDialog => Some(
                Dialog::new(tr!("Select Date"))
                    .with_child(self.date_range_form(ctx))
                    .on_close(ctx.link().callback(|_| Msg::CloseDialog)),
            ),
            ViewState::ShowSubscriptionNotice => Some(
                Dialog::new(tr!("No valid subscription"))
                    .with_child(
                        Column::new()
                            .padding(2)
                            .gap(1)
                            .with_child(proxmox_yew_comp::subscription_note(None))
                            .with_child(
                                Row::new().class(JustifyContent::FlexEnd).with_child(
                                    Button::new(tr!("OK"))
                                        .on_activate(ctx.link().callback(|_| Msg::CloseDialog)),
                                ),
                            ),
                    )
                    .on_close(ctx.link().callback(|_| Msg::CloseDialog)),
            ),
        };

        let fab = Fab::new("fa fa-calendar").on_activate(ctx.link().callback(|_| Msg::ShowDialog));

        let sub_notice = match self.subscription_result {
            Some(true) | None => None,
            Some(false) => Some(
                Column::new()
                    .class("pwt-default-colors")
                    .class(ColorScheme::Surface)
                    .class(JustifyContent::Stretch)
                    .padding(1)
                    .with_child(
                        Button::new(tr!("No valid Subscription"))
                            .icon_class("fa fa-exclamation-triangle")
                            .class("pwt-button-text")
                            .on_activate(ctx.link().callback(|_| Msg::ShowSubscriptionNotice)),
                    ),
            ),
        };

        Scaffold::new()
            .application_bar(
                ApplicationBar::new()
                    .leading(
                        Image::new("/proxmox_logo.png")
                            .dark_mode_src("/proxmox_logo_white.png")
                            .height(30)
                            .class("pwt-navbar-brand"),
                    )
                    .title("Mail")
                    .with_action(ThemeModeSelector::new()),
            )
            .body(
                Column::new()
                    .class(FlexFit)
                    .with_child(content)
                    .with_optional_child(dialog)
                    .with_flex_spacer()
                    .with_optional_child(sub_notice),
            )
            .favorite_action_button(fab)
            .into()
    }
}

impl From<PageSpamList> for VNode {
    fn from(val: PageSpamList) -> Self {
        let comp = VComp::new::<PmgPageSpamList>(Rc::new(val), None);
        VNode::from(comp)
    }
}
