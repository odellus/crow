use dioxus::prelude::*;
use ui::Chat;

#[component]
pub fn Home() -> Element {
    rsx! {
        Chat {}
    }
}
