use std::{
    fmt::{Display, format},
    str::FromStr,
};

use dioxus::{
    CapturedError,
    core::{IntoAttributeValue, anyhow, bail},
    document::eval,
    prelude::*,
};
use reqwasm::http::Request;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus_logger::initialize_default();
    launch(Controller);
}

macro_rules! twnum {
    ($num:expr) => {
        TailwindNumber::Number($num)
    };
    ($top:expr , $bottom: expr) => {
        TailwindNumber::Fraction(($top, $bottom))
    };
}

// --- API Logic (Unchanged) ---
async fn send_command((action, status): (&str, &str)) {
    let params = format!("cmd={action}&status={status}");
    let resp = Request::post("/controller")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(&params)
        .send()
        .await;

    match resp {
        Ok(x) if x.status() != 200 => {
            error!("Failed: {} code: {}", params, x.status());
        }
        Err(err) => {
            info!("Error: {} - {}", params, err);
        }
        _ => {}
    }
}

// --- STYLES: Animated Mesh Background & Glass Utilities ---
const GLOBAL_STYLES: &str = r#"
    @keyframes blob {
        0% { transform: translate(0px, 0px) scale(1); }
        33% { transform: translate(30px, -50px) scale(1.1); }
        66% { transform: translate(-20px, 20px) scale(0.9); }
        100% { transform: translate(0px, 0px) scale(1); }
    }
    .animate-blob {
        animation: blob 7s infinite;
    }
    .animation-delay-2000 {
        animation-delay: 2s;
    }
    .animation-delay-4000 {
        animation-delay: 4s;
    }
    /* The "Gradient Stroke" effect wrapper */
    .glass-border-gradient {
        background: linear-gradient(135deg, rgba(255,255,255,0.6) 0%, rgba(255,255,255,0.1) 50%, rgba(255,255,255,0.05) 100%);
        padding: 1px; /* The thickness of the stroke */
        border-radius: 9999px;
    }
    /* The Inner Glass Fill */
    .glass-panel {
        background: rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.3);
    }
    /* The Floating Puck */
    .glass-puck {
        background: linear-gradient(135deg, rgba(255,255,255,0.4) 0%, rgba(255,255,255,0.1) 100%);
        backdrop-filter: blur(20px);
        border: 1px solid rgba(255, 255, 255, 0.5);
        box-shadow: 
            0 4px 6px -1px rgba(0, 0, 0, 0.1), 
            0 2px 4px -1px rgba(0, 0, 0, 0.06),
            inset 0 0 20px rgba(255,255,255,0.2);
    }

    /* Custom Range Slider Styling */
    input[type=range]::-webkit-slider-thumb {
        -webkit-appearance: none;
        height: 15px;
        width: 15px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.9);
        border: 2px solid rgba(255, 255, 255, 0.5);
        box-shadow: 0 0 10px rgba(255, 255, 255, 0.5);
        margin-top: -3px; /* You might need to adjust this based on browser */
    }
    input[type=range]::-moz-range-thumb {
        height: 15px;
        width: 15px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.9);
        border: 2px solid rgba(255, 255, 255, 0.5);
        box-shadow: 0 0 10px rgba(255, 255, 255, 0.5);
    }
    input[type=range]::-webkit-slider-runnable-track {
        height: 8px;
        border-radius: 4px;
        background: rgba(0, 0, 0, 0.3);
    }
"#;

#[component]
fn Controller() -> Element {
    let toggle_fullscreen = move |_| {
        let js_script = r"
            var elem = document.documentElement;
            if (!document.fullscreenElement) {
                elem.requestFullscreen().catch(err => {});
            } else {
                document.exitFullscreen();
            }
        ";
        let _ = eval(js_script);
    };

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        style { "{GLOBAL_STYLES}" }

        // --- Main Container with Big Sur Style Background ---
        div { class: "relative flex not-sm:flex-col items-center justify-between h-screen w-screen overflow-hidden bg-slate-900 gap-10 p-6 sm:px-12 touch-none select-none text-white",

            // 1. Animated Background Blobs
            div { class: "absolute top-0 left-0 w-full h-full overflow-hidden z-10",
                // div { class: "absolute top-0 -left-4 w-screen h-screen bg-blue-300 rounded-full mix-blend-multiply filter blur-[128px] opacity-60 animate-blob" }
                // div { class: "absolute top-0 -right-4 w-96 h-96 bg-pink-500 rounded-full mix-blend-multiply filter blur-[128px] opacity-60 animate-blob animation-delay-2000" }
                // div { class: "absolute -bottom-32 left-20 w-96 h-96 bg-pink-500 rounded-full mix-blend-multiply filter blur-[128px] opacity-30 animate-blob animation-delay-2000" }
                // div { class: "absolute inset-0 bg-black/20" }
            }

            div {
                class: "absolute left-1/2 -translate-x-1/2 top-2 z-1000",
                // --- INSERT THE SLIDER HERE ---
                BlinkSlider {}
            }

            button {
                class: "absolute top-6 left-6 group flex items-center justify-center p-3 rounded-full glass-panel hover:bg-white/20 transition-all duration-300 z-50",
                onclick: toggle_fullscreen,
                span { class: "text-white/80 group-hover:text-white font-bold text-lg", "⛶" }
            }

            div { class: "flex flex-col items-center gap-6 justify-self-start z-10",
                span { class: "text-white/60 font-bold uppercase tracking-[0.2em] text-sm drop-shadow-md", "Movement" }
                AnalogJoystick {}

            }

            div { class: "justify-self-end flex justify-end gap-12 z-10",
                VerticalJoystick {
                    title: "Lift",
                    press_cmd: |cmd| match cmd {
                        Direction::Up => Some(("pull_up", "pressed")),
                        Direction::Down => Some(("pull_down", "pressed")),
                        _ => None,
                    },
                    release_cmd: |cmd| match cmd {
                        Direction::Up => Some(("pull_up", "released")),
                        Direction::Down => Some(("pull_down", "released")),
                        _ => None,
                    }
                }

                VerticalJoystick {
                    title: "Arm",
                    press_cmd: |cmd| match cmd {
                        Direction::Up => Some(("arm_up", "pressed")),
                        Direction::Down => Some(("arm_down", "pressed")),
                        _ => None,
                    },
                    release_cmd: |cmd| match cmd {
                        Direction::Up => Some(("arm_up", "released")),
                        Direction::Down => Some(("arm_down", "released")),
                        _ => None,
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Direction {
    Neutral,
    Up,
    Down,
    Left,
    Right,
}

// --- COMPONENT: Vertical Slider ---
#[component]
fn VerticalJoystick(
    title: &'static str,
    press_cmd: Callback<Direction, Option<(&'static str, &'static str)>>,
    release_cmd: Callback<Direction, Option<(&'static str, &'static str)>>,
) -> Element {
    let mut stick_pos = use_signal(|| 0.0f64);
    let mut drag_start = use_signal(|| None::<f64>);
    let mut active_dir = use_signal(|| Direction::Neutral);

    let max_radius = 60.0;
    let deadzone = 15.0;

    let direction_pressed = use_signal(|| false);

    // Logic implementation (same as before, just styling changed)
    let mut handle_move = move |client_y: f64| {
        if let Some(start) = drag_start()
            && !direction_pressed()
        {
            let delta_y = client_y - start;
            let vis_y = if delta_y.abs() > max_radius {
                let ratio = max_radius / delta_y.abs();
                delta_y * ratio
            } else {
                delta_y
            };

            stick_pos.set(vis_y);

            let new_dir = if delta_y.abs() < deadzone {
                Direction::Neutral
            } else if delta_y > 0.0 {
                Direction::Down
            } else {
                Direction::Up
            };

            let current = *active_dir.read();
            if new_dir != current {
                spawn(async move {
                    if let Some(cmd) = release_cmd(current) {
                        send_command(cmd).await;
                    }
                    if let Some(cmd) = press_cmd(new_dir) {
                        send_command(cmd).await;
                    }
                });
                active_dir.set(new_dir);
            }
        }
    };

    let mut handle_end = move || {
        drag_start.set(None);
        stick_pos.set(0.0);
        let current = *active_dir.read();
        if current != Direction::Neutral {
            spawn(async move {
                if let Some(cmd) = release_cmd(current) {
                    send_command(cmd).await;
                }
            });
            active_dir.set(Direction::Neutral);
        }
    };

    rsx! {
        div { class: "flex flex-col items-center gap-6",
            span { class: "text-white/60 font-bold uppercase tracking-[0.2em] text-sm drop-shadow-md", "{title}" }

            // 1. The Gradient Stroke Container
            div { class: "glass-border-gradient",
                // 2. The Inner Glass Track
                div {
                    class: "relative w-24 h-[240px] rounded-full glass-panel flex justify-center items-center overflow-hidden",

                    // Touch Handlers
                    onpointerdown: move |e| drag_start.set(Some(e.client_coordinates().y)),
                    onpointermove: move |e| handle_move(e.client_coordinates().y),
                    onpointerup: move |_| handle_end(),
                    onpointerleave: move |_| handle_end(),

                    for dir in [Direction::Up, Direction::Down] {
                        DirectionButton {
                            dir,
                            send_command_args: (dir.to_command(), "blink_once"),
                            direction_pressed,
                            gap: 3,
                            padding: 5,
                        }
                    }


                    // Visual Track Line
                    div { class: "absolute w-[2px] h-3/4 bg-white/5 rounded-full" }

                    // 3. The Interactive Puck
                    div {
                        class: "absolute w-16 h-16 rounded-full glass-puck flex items-center justify-center duration-800 ease-(--quick-easing)",
                        will_change: "transform",
                        transition_property: if drag_start().is_none() { "transform" } else { "none" },
                        transform: "translateY({stick_pos}px)",

                        // Icon
                        div { class: "text-white/90 text-4xl font-bold drop-shadow-lg",
                            "●"
                        }
                    }
                }
            }
        }
    }
}

// --- COMPONENT: 360 Analog Joystick ---
#[component]
fn AnalogJoystick() -> Element {
    let mut stick_pos = use_signal(|| (0.0f64, 0.0f64));
    let mut drag_start = use_signal(|| None::<(f64, f64)>);
    let mut active_dir = use_signal(|| Direction::Neutral);

    let max_radius = 50.0; // Visual restriction
    let deadzone = 10.0;

    let press_cmd = move |cmd| match cmd {
        Direction::Neutral => None,
        x => Some((x.to_command(), "pressed")),
    };
    let release_cmd = move |cmd| match cmd {
        Direction::Neutral => None,
        x => Some((x.to_command(), "released")),
    };

    let direction_pressed = use_signal(|| false);

    let mut handle_move = move |client_x: f64, client_y: f64| {
        if let Some(start) = drag_start()
            && !direction_pressed()
        {
            let delta_x = client_x - start.0;
            let delta_y = client_y - start.1;
            let distance = delta_x.hypot(delta_y);

            let (vis_x, vis_y) = if distance > max_radius {
                let ratio = max_radius / distance;
                (delta_x * ratio, delta_y * ratio)
            } else {
                (delta_x, delta_y)
            };

            stick_pos.set((vis_x, vis_y));

            let new_dir = if distance < deadzone {
                Direction::Neutral
            } else if delta_x.abs() > delta_y.abs() {
                if delta_x > 0.0 {
                    Direction::Right
                } else {
                    Direction::Left
                }
            } else if delta_y > 0.0 {
                Direction::Down
            } else {
                Direction::Up
            };

            let current = *active_dir.read();
            if new_dir != current {
                spawn(async move {
                    if let Some(cmd) = release_cmd(current) {
                        send_command(cmd).await;
                    }
                    if let Some(cmd) = press_cmd(new_dir) {
                        send_command(cmd).await;
                    }
                });
                active_dir.set(new_dir);
            }
        }
    };

    let mut handle_end = move || {
        drag_start.set(None);
        stick_pos.set((0.0, 0.0));
        let current = *active_dir.read();
        if current != Direction::Neutral {
            spawn(async move {
                if let Some(cmd) = release_cmd(current) {
                    send_command(cmd).await;
                }
            });
            active_dir.set(Direction::Neutral);
        }
    };

    rsx! {
        // 1. Gradient Stroke Container
        div { class: "glass-border-gradient",
            // 2. Inner Glass Base
            div {
                class: "relative size-[250px] rounded-full glass-panel flex items-center justify-center",

                onpointerdown: move |e| drag_start.set(Some((e.client_coordinates().x, e.client_coordinates().y))),
                onpointermove: move |e| handle_move(e.client_coordinates().x, e.client_coordinates().y),
                onpointerup: move |_| handle_end(),
                onpointerleave: move |_| handle_end(),

                for dir in [Direction::Up, Direction::Left, Direction::Down, Direction::Right] {
                    DirectionButton {
                        dir,
                        send_command_args: (dir.to_command(), "blink_once"),
                        direction_pressed,
                        gap: 0,
                        padding: 8,
                    }
                }

                // 3. The Interactive Puck
                div {
                    class: "absolute w-24 h-24 rounded-full glass-puck flex items-center justify-center duration-800 ease-(--quick-easing)",
                    will_change: "transform",
                    transition_property: if drag_start().is_none() { "transform" } else { "none" },
                    transform: "translate({stick_pos().0}px, {stick_pos().1}px)",

                    div { class: "text-white/90 text-4xl font-bold drop-shadow-lg",
                        "●"
                    }

                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TailwindNumber {
    Number(i32),
    Fraction((i32, i32)),
}

impl From<i32> for TailwindNumber {
    fn from(val: i32) -> Self {
        Self::Number(val)
    }
}

impl Display for TailwindNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(x) => {
                write!(f, "calc(var(--spacing) * {x})")
            }
            Self::Fraction((top, bottom)) => {
                write!(f, "calc({top}/{bottom} * 100)")
            }
        }
    }
}

impl IntoAttributeValue for TailwindNumber {
    fn into_value(self) -> dioxus_core::AttributeValue {
        dioxus_core::AttributeValue::Text(self.to_string())
    }
}

impl Direction {
    const fn to_command(self) -> &'static str {
        match self {
            Self::Up => "go_front",
            Self::Down => "go_back",
            Self::Left => "turn_left",
            Self::Right => "turn_right",
            Self::Neutral => unreachable!(),
        }
    }
}

#[component]
fn DirectionButton(
    dir: Direction,
    send_command_args: (&'static str, &'static str),
    direction_pressed: Signal<bool>,
    #[props(into)] gap: TailwindNumber,
    #[props(into)] padding: TailwindNumber,
) -> Element {
    let (style, string) = match dir {
        Direction::Up => (format!("top:{gap}"), "↑"),
        Direction::Down => (format!("bottom:{gap}"), "↓"),
        Direction::Left => (format!("left:{gap}"), "←"),
        Direction::Right => (format!("right:{gap}"), "→"),
        Direction::Neutral => bail!("Invalid Direction State"),
    };
    rsx! {
        button {
            class: "absolute size-[20px] flex items-center justify-center group",
            style,
            padding,
            onpointerdown: move |e| async move {
                send_command(send_command_args).await;
                direction_pressed.set(true);
            },
            onpointerup: move |e| {
                direction_pressed.set(false);
            },
            div { class: "text-white/30 group-active:text-white/90 text-3xl font-bold drop-shadow-lg",
                "{string}"
            }
        }
    }
}

#[component]
fn BlinkSlider() -> Element {
    let mut blink_len = use_signal(|| 75); // Default start value

    rsx! {
        div { class: "glass-border-gradient !rounded-2xl w-48", // Fixed width to match joystick roughly
            div { class: "glass-panel p-3 rounded-2xl flex flex-col gap-3",

                // Label and Value Display
                div { class: "flex justify-between items-center text-xs font-bold tracking-widest text-white/70 uppercase",
                    span { "Blink Rate" }
                    span { class: "text-white text-shadow", "{blink_len} ms" }
                }

                // The Slider Input
                input {
                    r#type: "range",
                    min: "50",
                    max: "150",
                    step: "5",
                    value: "{blink_len}",
                    class: "w-full h-2 bg-slate-700/50 rounded-lg appearance-none cursor-pointer accent-pink-500 hover:accent-pink-400 transition-all",

                    // Updates the UI text immediately while dragging
                    oninput: move |evt| async move {
                        let value = evt.value();
                        if let Ok(val) = value.parse::<i32>() {
                            send_command(("blink_rate", &value)).await;
                            blink_len.set(val);
                        }
                    },

                    // Sends the command only when the user releases the handle
                    onchange: move |evt| {
                        if let Ok(val) = evt.value().parse::<i32>() {
                            spawn(async move {
                                let val_str = val.to_string();
                                // Sends: cmd=set_blink_len&status=500
                                send_command(("set_blink_len", &val_str)).await;
                            });
                        }
                    }
                }
            }
        }
    }
}
