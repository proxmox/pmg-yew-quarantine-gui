use std::rc::Rc;

use anyhow::Error;
use serde_json::Value;

use yew::virtual_dom::{VComp, VNode};

use pwt::dom::get_system_prefer_dark_mode;
use pwt::prelude::*;
use pwt::state::{Theme, ThemeObserver};
use pwt::touch::{ApplicationBar, FabMenu, FabMenuEntry, Scaffold, SnackBar, SnackBarContextExt};
use pwt::widget::form::Checkbox;
use pwt::widget::{get_unique_element_id, FieldLabel, Row};

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
    DarkmodeFilter(bool), // on/off
    DarkmodeChange(bool), // on/off
}
pub struct PmgPageMailView {
    show_dark_mode_filter: bool,
    dark_mode_filter: bool,
    _theme_observer: ThemeObserver,
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

        let style = self
            .dark_mode_filter
            .then_some("filter: brightness(95%) invert(95%);");

        html! {
            <iframe {style} frameborder="0" width="100%" height="100%" sandbox="allow-same-origin"
                src={format!("/api2/htmlmail/quarantine/content?id={}", props.id)}>
            </iframe>
        }
    }
}

impl Component for PmgPageMailView {
    type Message = Msg;
    type Properties = PageMailView;

    fn create(_ctx: &Context<Self>) -> Self {
        let theme = Theme::load();
        let dark_mode_filter = match theme.mode {
            pwt::state::ThemeMode::System => get_system_prefer_dark_mode(),
            pwt::state::ThemeMode::Dark => true,
            pwt::state::ThemeMode::Light => false,
        };

        let _theme_observer = ThemeObserver::new(
            _ctx.link()
                .callback(|(_, dark_mode)| Msg::DarkmodeChange(dark_mode)),
        );

        Self {
            dark_mode_filter,
            show_dark_mode_filter: dark_mode_filter,
            _theme_observer,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ActionResult(result) => {
                if let Err(err) = result {
                    ctx.link()
                        .show_snackbar(SnackBar::new().message(err.to_string()));
                }
                true
            }
            Msg::DarkmodeFilter(dark_mode_filter) => {
                let changed = self.dark_mode_filter != dark_mode_filter;
                self.dark_mode_filter = dark_mode_filter;
                changed
            }
            Msg::DarkmodeChange(dark_mode) => {
                let changed = self.show_dark_mode_filter != dark_mode;
                self.show_dark_mode_filter = dark_mode;
                // deactivate the dark mode filter if we don't show the checkbox
                if !self.show_dark_mode_filter {
                    self.dark_mode_filter = false;
                }
                changed
            }
        }
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

        let mut app_bar = ApplicationBar::new().title(tr!("Preview"));

        if self.show_dark_mode_filter {
            let id = get_unique_element_id();
            app_bar.add_action(
                Row::new()
                    .class(pwt::css::AlignItems::Center)
                    .gap(1)
                    .with_child(FieldLabel::new(tr!("Dark-mode filter")).id(id.clone()))
                    .with_child(
                        Checkbox::new()
                            .label_id(id)
                            .checked(self.dark_mode_filter)
                            .on_change(ctx.link().callback(Msg::DarkmodeFilter)),
                    ),
            );
        }

        Scaffold::new()
            .application_bar(app_bar)
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
