use std::fmt::Display;

use dioxus::{
    core::{IntoAttributeValue, bail},
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
            error!("Error: {} - {}", params, err);
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

#[derive(Copy, Clone, PartialEq)]
enum ActiveTab {
    Control,
    Settings,
}

#[component]
fn Controller() -> Element {
    let mut active_tab = use_signal(|| ActiveTab::Control);
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
        div { class: "relative flex not-sm:flex-col items-center justify-between h-screen w-screen overflow-hidden bg-slate-900 gap-10 p-6 sm:px-12 pt-24 touch-none select-none text-white",

            // --- TOP HEADER BAR ---
            div { class: "absolute top-6 left-6 right-6 flex justify-between items-center z-50",
                // Fullscreen Toggle
                button {
                    class: "group flex items-center justify-center p-3 rounded-xl glass-panel hover:bg-white/20 transition-all duration-300",
                    onclick: toggle_fullscreen,
                    span { class: "text-white/80 group-hover:text-white font-bold text-lg", "⛶" }
                }

                // Tab Selector
                div { class: "glass-border-gradient !rounded-full",
                    div { class: "glass-panel flex p-1 rounded-full",
                        button {
                            class: format!("px-6 py-2 rounded-full text-sm font-bold transition-all {}",
                                if active_tab() == ActiveTab::Control { "bg-white/20 text-white shadow-inner" } else { "text-white/40 hover:text-white/60" }),
                            onclick: move |_| active_tab.set(ActiveTab::Control),
                            "Control"
                        }
                        button {
                            class: format!("px-6 py-2 rounded-full text-sm font-bold transition-all {}",
                                if active_tab() == ActiveTab::Settings { "bg-white/20 text-white shadow-inner" } else { "text-white/40 hover:text-white/60" }),
                            onclick: move |_| active_tab.set(ActiveTab::Settings),
                            "Settings"
                        }
                    }
                }
            }


            div {
                class: "w-full h-full flex flex-col items-center justify-start gap-8 touch-pan-y overflow-y-auto",
                hidden: active_tab() != ActiveTab::Settings,
                h2 { class: "text-2xl font-light tracking-widest text-white/50 uppercase", "System Settings" }

                div {
                    class: "w-full max-w-xl",
                    BlinkSlider {}
                }
                div {
                    class: "w-full max-w-xl",
                    PwmSlider {}
                }
                div {
                    class: "w-full max-w-xl",
                    FrequencySlider {}
                }

                div { class: "text-center opacity-20 text-xs mt-10",
                    p { "Connected to: robot-controller-v2" }
                    p { "Firmware: 0.0.1-alpha.0" }
                }
            }

            button {
                class: "absolute top-6 left-6 group flex items-center justify-center p-3 rounded-full glass-panel hover:bg-white/20 transition-all duration-300 z-50",
                onclick: toggle_fullscreen,
                span { class: "text-white/80 group-hover:text-white font-bold text-lg", "⛶" }
            }

            div {
                class: "contents",
                hidden: active_tab() != ActiveTab::Control,
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
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Direction {
    Neutral,
    Up,
    Down,
    Left,
    Right,
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
    fn icon(self) -> Result<&'static str> {
        Ok(match self {
            Self::Up => "↑",
            Self::Down => "↓",
            Self::Left => "←",
            Self::Right => "→",
            Self::Neutral => bail!("ain't got an icon"),
        })
    }
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

    // Logic implementation (same as before, just styling changed)
    let mut handle_move = move |client_y: f64| {
        if let Some(start) = drag_start() {
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
        spawn(async move {
            if let Some(cmd) = release_cmd(current) {
                send_command(cmd).await;
            }
        });
        active_dir.set(Direction::Neutral);
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
                    ontouchstart: move |e| {
                        if let Some(t) = e.data.touches().first() {
                            drag_start.set(Some(t.client_coordinates().y));
                        }
                    },
                    ontouchmove: move |e| {
                        if let Some(t) = e.data.touches().first() {
                            handle_move(t.client_coordinates().y);
                        }
                    },
                    ontouchend: move |_| handle_end(),

                    for dir in [Direction::Up, Direction::Down] {
                        DirectionButton {
                            dir,
                            send_command_args: (press_cmd(dir).context("well this has gone horribly wrong(state)")?.0, "blink_once"),
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

    let mut handle_move = move |client_x: f64, client_y: f64| {
        if let Some(start) = drag_start() {
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
        spawn(async move {
            if let Some(cmd) = release_cmd(current) {
                send_command(cmd).await;
            }
        });
        active_dir.set(Direction::Neutral);
    };

    rsx! {
        // 1. Gradient Stroke Container
        div { class: "glass-border-gradient",
            // 2. Inner Glass Base
            div {
                class: "relative size-[250px] rounded-full glass-panel flex items-center justify-center",

                ontouchstart: move |e| {
                    if let Some(t) = e.data.touches().first() {
                        let cords = t.client_coordinates();
                        drag_start.set(Some((cords.x, cords.y)));
                    }
                },
                ontouchmove: move |e| {
                    if let Some(t) = e.touches().first() {
                        let cords = t.client_coordinates();
                        handle_move(cords.x, cords.y);
                    }
                },
                ontouchend: move |_| handle_end(),

                for dir in [Direction::Up, Direction::Left, Direction::Down, Direction::Right] {
                    DirectionButton {
                        dir,
                        send_command_args: (dir.to_command(), "blink_once"),
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

#[component]
fn DirectionButton(
    dir: Direction,
    send_command_args: (&'static str, &'static str),
    #[props(into)] gap: TailwindNumber,
    #[props(into)] padding: TailwindNumber,
) -> Element {
    let style = match dir {
        Direction::Up => format!("top:{gap}"),
        Direction::Down => format!("bottom:{gap}"),
        Direction::Left => format!("left:{gap}"),
        Direction::Right => format!("right:{gap}"),
        Direction::Neutral => bail!("Invalid Direction State"),
    };

    rsx! {
        button {
            class: "absolute size-[20px] flex items-center justify-center group",
            style,
            padding,
            ontouchstart: move |e| async move {
                send_command(send_command_args).await;
            },
            div { class: "text-white/30 group-active:text-white/90 text-3xl font-bold drop-shadow-lg",
                {dir.icon()?}
            }
        }
    }
}

#[component]
fn BlinkSlider() -> Element {
    rsx! {
        CustomSlider {
            left: "fast (50ms)",
            right: "slow (150ms)",
            step: 5,
            title: "Blink Rate",
            unit: "ms",
            details: "Adjust the strobe frequency for signals",
            default_value: 75,
            command_type: "blink_rate",
            min: 50,
            max: 150
        }
    }
}

#[component]
fn PwmSlider() -> Element {
    rsx! {
        CustomSlider {
            left: "none(0%)",
            right: "full(100%)",
            title: "PWM Percentage",
            step: 5,
            unit: "%",
            details: "Adjust the PWM duty cycle using percentages",
            default_value: 100,
            command_type: "pwm_percentage",
            min: 20,
            max: 100
        }
    }
}

#[component]
fn FrequencySlider() -> Element {
    let max = 48000;
    let min = 500;
    rsx! {
        CustomSlider {
            left: "low({min})",
            right: "high({max})",
            step: 500,
            title: "PWM Frequency",
            unit: "Hz",
            details: "A higher frequency produces less hum, but lesser strength",
            default_value: 8000,
            command_type: "frequency_kilohertz",
            min,
            max
        }
    }
}

#[component]
fn CustomSlider(
    default_value: i32,
    unit: String,
    title: String,
    details: String,
    command_type: ReadSignal<String>,
    left: String,
    right: String,
    step: i32,
    min: i32,
    max: i32,
) -> Element {
    let mut blink_len = use_signal(|| default_value);

    rsx! {
        div { class: "glass-border-gradient !rounded-3xl w-full",
            div { class: "glass-panel p-8 rounded-3xl flex flex-col gap-6",
                div { class: "flex justify-between items-end",
                    div {
                        h3 { class: "text-xl font-bold text-white", "{title}" }
                        p { class: "text-sm text-white/40", "{details}" }
                    }
                    span { class: "text-4xl font-mono font-bold text-pink-500", "{blink_len}{unit}" }
                }

                input {
                    r#type: "range",
                    min,
                    max,
                    step,
                    value: "{blink_len}",
                    class: "w-full h-2 bg-slate-700/50 rounded-lg appearance-none cursor-pointer accent-pink-500 hover:accent-pink-400 transition-all",
                    oninput: move |evt| {
                        if let Ok(val) = evt.value().parse::<i32>() {
                            blink_len.set(val);
                            spawn(async move {
                                send_command((&command_type(), &val.to_string())).await;
                            });
                        }
                    },
                }

                div { class: "flex justify-between text-[10px] font-bold text-white/20 uppercase tracking-tighter",
                    span { {left} }
                    span { {right} }
                }
            }
        }
    }
}
