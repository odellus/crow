use dioxus::prelude::*;

use views::{Blog, Home, SessionDetail, Sessions};

mod views;

#[cfg(feature = "server")]
mod server;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Sessions {},
    #[route("/session/:session_id")]
    SessionDetail { session_id: String },
    #[route("/home")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    #[cfg(feature = "server")]
    {
        // For now, use default launch
        // TODO: Integrate custom REST routes with LaunchBuilder
        // See: crow/packages/api/src/bin/crow-serve.rs for standalone REST server
        dioxus::launch(App);
    }

    #[cfg(not(feature = "server"))]
    {
        dioxus::launch(App);
    }
}

#[component]
fn App() -> Element {
    // Build cool things ✌️

    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Router::<Route> {}
    }
}
