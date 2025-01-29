use chrono::{DateTime, Local};
use gloo_timers::callback::Interval;
use web_sys::window;
use yew::prelude::*;

#[derive(Clone, Debug)]
struct Task {
    description: String,
    completed_at: DateTime<Local>,
    duration: i32,
}

pub struct PomodoroTimer {
    time: i32,
    running: bool,
    interval: Option<Interval>,
    current_task: String,
    completed_tasks: Vec<Task>,
    initial_time: i32,
    markdown_visible: bool,
    markdown_content: String,
}

pub enum Msg {
    Start,
    Stop,
    Reset,
    Tick,
    UpdateTask(String),
    CompleteTask,
    ExportTasks,
    CopyToClipboard,
    HideMarkdown,
}

impl Component for PomodoroTimer {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            time: 1500,
            initial_time: 1500,
            running: false,
            interval: None,
            current_task: String::new(),
            completed_tasks: Vec::new(),
            markdown_visible: false,
            markdown_content: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Start => {
                if !self.running {
                    let link = ctx.link().clone();
                    self.initial_time = self.time;
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
            Msg::Reset => {
                self.time = 1500;
                self.initial_time = 1500;
                self.running = false;
                self.interval = None;
                true
            }
            Msg::Tick => {
                if self.time > 0 {
                    self.time -= 1;
                    if self.time == 0 && !self.current_task.is_empty() {
                        ctx.link().send_message(Msg::CompleteTask);
                    }
                    true
                } else {
                    self.running = false;
                    self.interval = None;
                    true
                }
            }
            Msg::UpdateTask(task) => {
                self.current_task = task;
                true
            }
            Msg::CompleteTask => {
                if !self.current_task.is_empty() {
                    let actual_duration = self.initial_time - self.time;
                    self.completed_tasks.push(Task {
                        description: self.current_task.clone(),
                        completed_at: Local::now(),
                        duration: actual_duration,
                    });
                    self.current_task.clear();
                    self.running = false;
                    self.interval = None;
                    self.time = 1500;
                    self.initial_time = 1500;
                }
                true
            }
            Msg::ExportTasks => {
                let mut markdown = String::new();
                let mut current_date = None;

                for task in &self.completed_tasks {
                    let task_date = task.completed_at.date_naive();
                    if current_date != Some(task_date) {
                        if let Some(_) = current_date {
                            markdown.push_str("\n");
                        }
                        markdown.push_str(&format!("## {}\n", task_date.format("%Y-%m-%d")));
                        current_date = Some(task_date);
                    }
                    let duration_minutes = task.duration / 60;
                    markdown.push_str(&format!(
                        "- {} (作業時間: {}分, 完了時刻: {})\n",
                        task.description,
                        duration_minutes,
                        task.completed_at.format("%H:%M:%S")
                    ));
                }

                self.markdown_content = markdown;
                self.markdown_visible = true;
                true
            }
            Msg::CopyToClipboard => {
                if let Some(window) = window() {
                    let navigator = window.navigator();
                    let clipboard = navigator.clipboard();
                    let _ = clipboard.write_text(&self.markdown_content);
                }
                true
            }
            Msg::HideMarkdown => {
                self.markdown_visible = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let minutes = self.time / 60;
        let seconds = self.time % 60;
        let is_running = self.running;
        let tasks_available = !self.completed_tasks.is_empty();
        let has_current_task = !self.current_task.is_empty();

        html! {
            <div class="p-4 max-w-2xl mx-auto">
                <h1 class="text-2xl font-bold mb-4 text-center">{"ポモドーロタイマー"}</h1>
                <div class="space-y-6">
                    <div class="text-center">
                        <p class="text-5xl font-mono mb-4">{format!("{:02}:{:02}", minutes, seconds)}</p>
                        <div class="space-x-4">
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
                            <button
                                onclick={ctx.link().callback(|_| Msg::Reset)}
                                class="px-6 py-2 bg-gray-500 text-white rounded hover:bg-gray-600 focus:outline-none"
                            >
                                {"リセット"}
                            </button>
                        </div>
                    </div>

                    <div class="mt-8">
                        <h2 class="text-xl font-bold mb-2">{"現在のタスク"}</h2>
                        <div class="space-y-2">
                            <input
                                type="text"
                                value={self.current_task.clone()}
                                onchange={ctx.link().callback(|e: Event| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    Msg::UpdateTask(input.value())
                                })}
                                class="w-full px-3 py-2 border rounded focus:outline-none focus:border-blue-500"
                                placeholder="タスクを入力してください"
                            />
                            if has_current_task {
                                <button
                                    onclick={ctx.link().callback(|_| Msg::CompleteTask)}
                                    class="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 focus:outline-none"
                                >
                                    {"タスクを完了"}
                                </button>
                            }
                        </div>
                    </div>

                    <div class="mt-4">
                        <h2 class="text-xl font-bold mb-2">{"完了したタスク"}</h2>
                        <div class="space-y-2">
                            {for self.completed_tasks.iter().rev().take(5).map(|task| {
                                let duration_minutes = task.duration / 60;
                                html! {
                                    <div class="p-2 bg-gray-100 rounded">
                                        <p class="text-sm">{&task.description}</p>
                                        <p class="text-xs text-gray-500">
                                            {format!("作業時間: {}分 ({})",
                                                duration_minutes,
                                                task.completed_at.format("%Y-%m-%d %H:%M:%S"))}
                                        </p>
                                    </div>
                                }
                            })}
                        </div>
                        if tasks_available {
                            <button
                                onclick={ctx.link().callback(|_| Msg::ExportTasks)}
                                class="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 focus:outline-none"
                            >
                                {"タスクをエクスポート"}
                            </button>
                        }
                    </div>

                    if self.markdown_visible {
                        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4">
                            <div class="bg-white rounded-lg p-6 w-full max-w-2xl">
                                <div class="flex justify-between items-center mb-4">
                                    <h3 class="text-lg font-bold">{"マークダウン形式のタスク履歴"}</h3>
                                    <button
                                        onclick={ctx.link().callback(|_| Msg::HideMarkdown)}
                                        class="text-gray-500 hover:text-gray-700"
                                    >
                                        {"✕"}
                                    </button>
                                </div>
                                <pre class="bg-gray-100 p-4 rounded mb-4 overflow-auto max-h-96 font-mono text-sm whitespace-pre-wrap">
                                    {&self.markdown_content}
                                </pre>
                                <button
                                    onclick={ctx.link().callback(|_| Msg::CopyToClipboard)}
                                    class="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 focus:outline-none"
                                >
                                    {"クリップボードにコピー"}
                                </button>
                            </div>
                        </div>
                    }
                </div>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<PomodoroTimer>::new().render();
}
