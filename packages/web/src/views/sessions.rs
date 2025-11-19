use api::{list_sessions, Session};
use dioxus::prelude::*;

/// Sessions view - shows list of all sessions in sidebar + main content area
#[component]
pub fn Sessions() -> Element {
    let sessions = use_resource(|| async move { list_sessions().await });

    rsx! {
        div { class: "flex h-screen bg-gray-900",
            // Sidebar
            SessionSidebar { sessions: sessions }

            // Main content area - will show "Select a session" or session detail
            div { class: "flex-1 flex items-center justify-center text-gray-400",
                "Select a session to start chatting"
            }
        }
    }
}

#[component]
fn SessionSidebar(
    sessions: Resource<Result<Vec<Session>, dioxus::prelude::ServerFnError>>,
) -> Element {
    let mut is_expanded = use_signal(|| true);

    rsx! {
        div {
            class: if is_expanded() {
                "w-72 border-r border-gray-700 flex flex-col bg-gray-800"
            } else {
                "w-16 border-r border-gray-700 flex flex-col bg-gray-800"
            },

            // Header with toggle button
            div { class: "p-4 border-b border-gray-700 flex justify-between items-center",
                if is_expanded() {
                    h2 { class: "text-white font-semibold", "Sessions" }
                }
                button {
                    class: "text-gray-400 hover:text-white",
                    onclick: move |_| is_expanded.set(!is_expanded()),
                    if is_expanded() { "◀" } else { "▶" }
                }
            }

            // New Session Button
            div { class: "p-4",
                button {
                    class: "w-full bg-blue-600 hover:bg-blue-700 text-white py-2 px-4 rounded-lg font-semibold transition",
                    onclick: move |_| {
                        // TODO: Create new session and navigate
                    },
                    if is_expanded() {
                        "+ New Session"
                    } else {
                        "+"
                    }
                }
            }

            // Session List
            div { class: "flex-1 overflow-y-auto",
                match sessions.read_unchecked().as_ref() {
                    Some(Ok(session_list)) => rsx! {
                        for session in session_list {
                            SessionItem { session: session.clone(), is_expanded: is_expanded() }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div { class: "p-4 text-red-400 text-sm",
                            "Error: {e}"
                        }
                    },
                    None => rsx! {
                        div { class: "p-4 text-gray-400 text-sm",
                            "Loading sessions..."
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SessionItem(session: Session, is_expanded: bool) -> Element {
    let formatted_time = format_time(session.time.updated);
    let title = if session.title.is_empty() {
        "Untitled".to_string()
    } else {
        session.title.clone()
    };
    let files_changed = session.summary.as_ref().map(|s| s.files).unwrap_or(0);

    rsx! {
        div {
            class: "px-4 py-3 hover:bg-gray-700 cursor-pointer border-b border-gray-700/50 transition",
            onclick: move |_| {
                // TODO: Navigate to session detail
            },

            {if is_expanded {
                rsx! {
                    div { class: "flex justify-between items-start mb-1",
                        div { class: "text-white text-sm font-medium truncate flex-1",
                            "{title}"
                        }
                    }
                    div { class: "text-gray-400 text-xs",
                        "{formatted_time}"
                    }
                    if session.summary.is_some() {
                        div { class: "text-gray-500 text-xs mt-1",
                            "{files_changed} files changed"
                        }
                    }
                }
            } else {
                rsx! {
                    div { class: "text-white text-xs text-center",
                        "💬"
                    }
                }
            }}
        }
    }
}

fn format_time(timestamp: u64) -> String {
    // Convert milliseconds to readable format
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let diff = now.saturating_sub(timestamp);
    let seconds = diff / 1000;

    if seconds < 60 {
        "Just now".to_string()
    } else if seconds < 3600 {
        format!("{} min ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{} hr ago", seconds / 3600)
    } else {
        format!("{} days ago", seconds / 86400)
    }
}
