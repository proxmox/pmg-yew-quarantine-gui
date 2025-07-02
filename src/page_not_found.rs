use pwt::prelude::*;
use pwt::touch::{ApplicationBar, Scaffold};
use pwt::widget::error_message;

#[function_component]
pub fn PageNotFound() -> Html {
    Scaffold::new()
        .application_bar(ApplicationBar::new().title(tr!("Not found")))
        .body(error_message(&tr!("page not found")))
        .into()
}
