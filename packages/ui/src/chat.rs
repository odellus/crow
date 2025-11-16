use api::{Part, ToolState};
use dioxus::prelude::*;

/// Main chat component - displays conversation history and input
#[component]
pub fn Chat() -> Element {
    let mut messages = use_signal(|| Vec::<String>::new());
    let mut parts = use_signal(|| Vec::<Part>::new());
    let mut input_value = use_signal(|| String::new());

    rsx! {
        div { class: "flex flex-col h-screen bg-gray-900",
            // Chat history
            div { class: "flex-1 overflow-y-auto p-4 space-y-4",
                // User messages
                for msg in messages.read().iter() {
                    UserMessage { content: msg.clone() }
                }

                // Assistant response parts
                if !parts.read().is_empty() {
                    AssistantMessage { parts: parts.read().clone() }
                }
            }

            // Input area
            div { class: "border-t border-gray-700 p-4",
                form {
                    onsubmit: move |event| async move {
                        event.prevent_default();
                        web_sys::console::log_1(&"Form submitted!".into());
                        let content = input_value.read().clone();
                        web_sys::console::log_1(&format!("Content: '{}'", content).into());
                        if !content.is_empty() {
                            web_sys::console::log_1(&"Content not empty, adding message".into());
                            // Add user message
                            messages.write().push(content.clone());
                            input_value.set(String::new());

                            // Get response from server
                            web_sys::console::log_1(&"About to call send_message()".into());
                            let result = api::send_message(content.clone()).await;
                            web_sys::console::log_1(&format!("send_message returned: {:?}", result.is_ok()).into());

                            match result {
                                Ok(response_parts) => {
                                    web_sys::console::log_1(&format!("Success! Got {} parts", response_parts.len()).into());
                                    parts.set(response_parts);
                                }
                                Err(e) => {
                                    web_sys::console::log_1(&format!("Error sending message: {:?}", e).into());
                                }
                            }
                        } else {
                            web_sys::console::log_1(&"Content is empty".into());
                        }
                    },
                    div { class: "flex gap-2",
                        input {
                            r#type: "text",
                            class: "flex-1 bg-gray-800 text-white border border-gray-600 rounded-lg px-4 py-2 focus:outline-none focus:border-blue-500",
                            placeholder: "Type your message...",
                            value: "{input_value}",
                            oninput: move |event| input_value.set(event.value()),
                        }
                        button {
                            r#type: "submit",
                            class: "bg-blue-600 hover:bg-blue-700 text-white px-6 py-2 rounded-lg font-semibold transition",
                            "Send"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn UserMessage(content: String) -> Element {
    rsx! {
        div { class: "flex justify-end",
            div { class: "bg-blue-600 text-white rounded-lg px-4 py-2 max-w-2xl",
                "{content}"
            }
        }
    }
}

#[component]
fn AssistantMessage(parts: Vec<Part>) -> Element {
    rsx! {
        div { class: "flex justify-start",
            div { class: "bg-gray-800 text-white rounded-lg px-4 py-3 max-w-2xl space-y-3",
                for part in parts {
                    PartRenderer { part: part }
                }
            }
        }
    }
}

#[component]
fn PartRenderer(part: Part) -> Element {
    match part {
        Part::Text { text, .. } => rsx! {
            div { class: "text-gray-100",
                "{text}"
            }
        },
        Part::Thinking { text, .. } => rsx! {
            div { class: "text-gray-400 italic text-sm border-l-2 border-gray-600 pl-3",
                "💭 {text}"
            }
        },
        Part::Tool { tool, state, .. } => rsx! {
            ToolRenderer { tool: tool, state: state }
        },
        Part::File { filename, url, .. } => rsx! {
            div { class: "bg-gray-700 rounded p-2",
                "📎 "
                a {
                    href: "{url}",
                    class: "text-blue-400 hover:underline",
                    "{filename.as_ref().map(|s| s.as_str()).unwrap_or(\"file\")}"
                }
            }
        },
    }
}

#[component]
fn ToolRenderer(tool: String, state: ToolState) -> Element {
    match state {
        ToolState::Pending { raw, .. } => rsx! {
            div { class: "bg-yellow-900/30 border border-yellow-700 rounded p-3",
                div { class: "text-yellow-400 font-semibold mb-1",
                    "🔧 {tool} (pending)"
                }
                pre { class: "text-xs text-gray-300 overflow-x-auto",
                    "{raw}"
                }
            }
        },
        ToolState::Running { title, .. } => rsx! {
            div { class: "bg-blue-900/30 border border-blue-700 rounded p-3",
                div { class: "text-blue-400 font-semibold",
                    "⚙️ {tool} - {title.as_ref().map(|s| s.as_str()).unwrap_or(\"running...\")}"
                }
            }
        },
        ToolState::Completed {
            title,
            output,
            time,
            ..
        } => rsx! {
            div { class: "bg-green-900/30 border border-green-700 rounded p-3",
                div { class: "text-green-400 font-semibold mb-2",
                    "✅ {tool} - {title}"
                    if let Some(end) = time.end {
                        span { class: "text-gray-500 text-xs ml-2",
                            "({end - time.start}ms)"
                        }
                    }
                }
                pre { class: "text-sm text-gray-300 bg-gray-950 p-2 rounded overflow-x-auto",
                    "{output}"
                }
            }
        },
        ToolState::Error { error, .. } => rsx! {
            div { class: "bg-red-900/30 border border-red-700 rounded p-3",
                div { class: "text-red-400 font-semibold mb-1",
                    "❌ {tool} failed"
                }
                div { class: "text-red-300 text-sm",
                    "{error}"
                }
            }
        },
    }
}
