use gloo_timers::callback::Timeout;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    components::{timer_display::TimerDisplay, timer_controls::TimerControls},
    helpers::format_time,
};


// Defines an async Rust function to call Tauri, used for updating the system tray state.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct SetTitleArgs<'a> {
    title: &'a str,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TimerState {
    Paused,
    Running,
    Break,
}

pub fn get_tray_title(timer_state: TimerState, timer_duration: u32, session_length: u32) -> String {
    match timer_state {
        TimerState::Paused => String::from("Paused"),
        TimerState::Running => {
            if timer_duration >= session_length {
                return format!("Finished session: {}", format_time(timer_duration));
            }
            return format!(
                "In session: {}",
                format_time(session_length - timer_duration)
            );
        }
        TimerState::Break => {
            if timer_duration >= session_length {
                return format!("Finished break: {}", format_time(timer_duration));
            }
            return format!("Break: {}", format_time(session_length - timer_duration));
        }
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let session_length = use_state(|| 25 * 60); // Default 25 minutes
    let timer_duration = use_state(|| 0);
    let timer_state = use_state(|| TimerState::Paused);

    use_effect_with_deps(
        move |props| {
            let (timer_duration, timer_state, _) = props.clone();

            let timeout = Timeout::new(1_000, move || {
                if *timer_state != TimerState::Paused {
                    timer_duration.set(*timer_duration + 1);
                }
            });

            let (timer_duration, timer_state, session_length) = props.clone();

            // Spawn a thread so that it can await the async call
            spawn_local(async move {
                let args = to_value(&SetTitleArgs {
                    title: get_tray_title(*timer_state, *timer_duration, *session_length).as_str(),
                })
                .unwrap();

                invoke("set_title", args).await;
            });

            move || {
                timeout.cancel();
            }
        },
        (
            timer_duration.clone(),
            timer_state.clone(),
            session_length.clone(),
        ),
    );

    html! {
        <div class={classes!("flex", "items-center", "justify-center", "flex-col", "h-screen")}>
            <TimerDisplay timer_state={timer_state.clone()} timer_duration={timer_duration.clone()} session_length={session_length.clone()} />
            <TimerControls session_length={session_length.clone()} timer_state={timer_state.clone()} timer_duration={timer_duration.clone()} />
        </div>
    }
}
