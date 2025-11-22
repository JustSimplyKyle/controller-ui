use dioxus::prelude::*;
use std::collections::HashMap;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus_logger::initialize_default();
    // Ideally wrap in a provider or generic layout
    launch(Controller);
}

// 1. The Logic: Handles the API call
async fn send_command(command: &str) {
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("command", command);

    // Using relative path "/controller" assumes same server
    // If cross-origin, use full URL: "http://192.168.2.1:8080/controller"
    let s = client
        .post("http://192.168.2.1:8080/controller")
        .form(&params)
        .send()
        .await;

    let p = s.and_then(|x| x.error_for_status());
    if let Err(e) = p {
        error!("{e}");
    } else {
        info!("Sent command: {}", command);
    }
}

#[component]
fn Controller() -> Element {
    // 2. The Style: Shadcn-like button classes
    // White bg, slate border, subtle shadow, hover effect, scale on click
    let btn_base = "
        flex items-center justify-center 
        h-14 w-14 rounded-md border border-slate-200 
        bg-white text-slate-950 shadow-sm 
        hover:bg-slate-100 hover:text-slate-900
        active:scale-95 transition-all duration-150 ease-in-out
        focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-400
    ";

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        // Container: Centered, Slate-50 background (Shadcn card style)
        div { class: "flex h-screen w-full items-center justify-center bg-slate-50 gap-6",

            // The Controller Card
            div { class: "flex flex-col items-center gap-6 rounded-xl border border-slate-200 bg-white p-8 shadow-lg",

                // Header
                div { class: "flex flex-col items-center gap-1.5",
                    div { class: "flex items-center gap-2 text-slate-900 font-semibold tracking-tight text-xl",
                        "移動控制"
                    }
                }
                // 3. The Layout: 3x3 Grid for the D-Pad
             div { class: "grid grid-cols-3 gap-3 *:select-none",

                    // --- ROW 1 ---
                    div {} // Empty Top-Left
                    button {
                        class: "{btn_base}",
                        ontouchstart: async move |_| {
                            send_command("go_front_pressed").await;
                        },
                        ontouchend: async move |_| {
                            send_command("go_front_released").await;
                        },

                        "↑" // Unicode U+2191
                    }
                    div {} // Empty Top-Right

                    // --- ROW 2 ---
                    button {
                        class: "{btn_base}",
                        ontouchstart: async move |_| {
                            send_command("turn_left_pressed").await;
                        },
                        ontouchend: async move |_| {
                            send_command("turn_left_released").await;
                        },
                        "←" // Unicode U+2190
                    }

                    // Center Button
                    div {}

                    button {
                        class: "{btn_base}",
                        ontouchstart: async move |_| {
                            send_command("turn_right_pressed").await;
                        },
                        ontouchend: async move |_| {
                            send_command("turn_right_released").await;
                        },
                        "→" // Unicode U+2192
                    }

                    // --- ROW 3 ---
                    div {} // Empty Bottom-Left
                    button {
                        class: "{btn_base}",
                        ontouchstart: async move |_| {
                            send_command("go_back_pressed").await;
                        },
                        ontouchend: async move |_| {
                            send_command("go_back_released").await;
                        },
                        "↓" // Unicode U+2193
                    }
                    div {} // Empty Bottom-Right
                }
            }
            div { class: "flex flex-col items-center gap-6 rounded-xl border border-slate-200 bg-white p-8 shadow-lg",

                // Header
                div { class: "flex flex-col items-center gap-1.5",
                    div { class: "flex items-center gap-2 text-slate-900 font-semibold tracking-tight text-xl",
                        "伸縮控制"
                    }
                }
                // 3. The Layout: 3x3 Grid for the D-Pad
             div { class: "grid grid-cols-1 gap-3 *:select-none",

                    button {
                        class: "{btn_base}",
                        ontouchstart: async move |_| {
                            send_command("pull_down_pressed").await;
                        },
                        ontouchend: async move |_| {
                            send_command("pull_down_released").await;
                        },

                        "↑" // Unicode U+2191
                    }
                    button {
                        class: "{btn_base}",
                        ontouchstart: async move |_| {
                            send_command("pull_up_pressed").await;
                        },
                        ontouchend: async move |_| {
                            send_command("pull_up_released").await;
                        },
                        "↓" // Unicode U+2193
                    }
                }
            }
        }
    }
}
