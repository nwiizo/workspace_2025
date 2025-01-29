use gloo_timers::callback::Interval;
use web_sys::console;
use yew::prelude::*;

pub struct PomodoroTimer {
    time: i32,
    running: bool,
    interval: Option<Interval>,
}

pub enum Msg {
    Start,
    Stop,
    Tick,
}

impl Component for PomodoroTimer {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            time: 1500,
            running: false,
            interval: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Start => {
                if !self.running {
                    let link = ctx.link().clone();
                    self.interval = Some(Interval::new(1000, move || {
                        link.send_message(Msg::Tick);
                    }));
                    self.running = true;
                }
                true
            }
            Msg::Stop => {
                self.running = false;
                self.interval = None;
                true
            }
            Msg::Tick => {
                if self.time > 0 {
                    console::log_1(&format!("Current time: {}", self.time).into());
                    self.time -= 1;
                    console::log_1(&format!("New time: {}", self.time).into());
                    true
                } else {
                    self.running = false;
                    self.interval = None;
                    true
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let minutes = self.time / 60;
        let seconds = self.time % 60;
        let is_running = self.running;

        html! {
            <div class="p-4">
                <h1 class="text-2xl font-bold mb-4 text-center">{"ポモドーロタイマー"}</h1>
                <div class="space-y-4">
                    <p class="text-4xl font-mono text-center">{format!("{:02}:{:02}", minutes, seconds)}</p>
                    <div class="text-center">
                        <button
                            onclick={ctx.link().callback(move |_| if is_running { Msg::Stop } else { Msg::Start })}
                            class={if is_running {
                                "px-6 py-2 bg-red-500 text-white rounded hover:bg-red-600 focus:outline-none"
                            } else {
                                "px-6 py-2 bg-green-500 text-white rounded hover:bg-green-600 focus:outline-none"
                            }}
                        >
                            {if is_running { "停止" } else { "開始" }}
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<PomodoroTimer>::new().render();
}
