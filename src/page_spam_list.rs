
use yew::prelude::*;
use yew_router::scope_ext::RouterScopeExt;

use pwt::prelude::*;
use pwt::touch::{Fab};
use pwt::widget::{Button, Column, Container, Dialog, Row};
use pwt::widget::form::Field;


use crate::{Route, SpamList, TopNavBar};

#[derive(Copy, Clone, PartialEq)]
pub enum ViewState {
    Normal,
    ShowDialog,
}
pub struct PageSpamList {
    state: ViewState,
}

pub enum Msg {
    Preview(String),
    ShowDialog,
    CloseDialog,
    ApplyDate,
}

impl PageSpamList {

    fn date_range_form(&self, ctx: &Context<Self>) -> Html {
        Column::new()
            .padding(2)
            .gap(1)
            //.attribute("style", "min-width:400px;min-height:300px;")
            .class("pwt-flex-fill")
            .with_child("From:")
            .with_child(
                Field::new()
                    .name("from")
                    .input_type("date")
            )
            .with_child("To:")
            .with_child(
                Field::new()
                    .name("to")
                    .input_type("date")
            )
            .with_child(
                Row::new()
                    .class("pwt-pt-2")
                    .with_flex_spacer()
                    .with_child(
                        Button::new("Apply")
                            .class("pwt-scheme-primary")
                            .onclick(ctx.link().callback(|_| Msg::ApplyDate))
                    )
            )
            .into()
    }
}

impl Component for PageSpamList {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            state: ViewState::Normal,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ShowDialog => {
                self.state = ViewState::ShowDialog;
                true
            }
            Msg::CloseDialog => {
                self.state = ViewState::Normal;
                true
            }
            Msg::ApplyDate => {
                // Fixme
                self.state = ViewState::Normal;
                true
            }
            Msg::Preview(id) => {
                //log::info!("Preview {id}");
                let navigator = ctx.link().navigator().unwrap();
                navigator.push(&Route::ViewMail { id: id.clone() });
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = SpamList::new()
            .on_preview(ctx.link().callback(|id| Msg::Preview(id)));

        let dialog = (self.state == ViewState::ShowDialog).then(||  {
            Dialog::new("Select Date")
                .with_child(self.date_range_form(ctx))
                .on_close(ctx.link().callback(|_| Msg::CloseDialog))
        });

        let fab = Container::new()
            .class("pwt-position-fixed")
            .class("pwt-right-2 pwt-bottom-4")
            .with_child(
                Fab::new("fa fa-calendar")
                    .class("pwt-scheme-primary")
                    .on_click(ctx.link().callback(|_| Msg::ShowDialog))
            );

        Column::new()
            .class("pwt-viewport")
            .with_child(TopNavBar::new())
            .with_child(content)
            .with_child(fab)
            .with_optional_child(dialog)
            .into()
    }
}
