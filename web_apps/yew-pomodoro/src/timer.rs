use crate::models::{AppState, Task, TimerConfig, TimerState};
use crate::storage::{load_state, save_state};
use gloo_timers::callback::Interval;
use web_sys::{window, HtmlSelectElement};
use yew::prelude::*;

pub struct PomodoroTimer {
    config: TimerConfig,
    time: i32,
    running: bool,
    interval: Option<Interval>,
    current_task: String,
    completed_tasks: Vec<Task>,
    state: TimerState,
    completed_pomodoros: i32,
    markdown_visible: bool,
    markdown_content: String,
    task_history: Vec<String>,
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
    TimerComplete,
    SelectPreviousTask(String),
}

impl PomodoroTimer {
    fn save_app_state(&self) {
        let app_state = AppState {
            completed_tasks: self.completed_tasks.clone(),
            task_history: self.task_history.clone(),
            completed_pomodoros: self.completed_pomodoros,
        };
        let _ = save_state(&app_state);
    }
}

impl Component for PomodoroTimer {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let config = TimerConfig::default();

        // Load state from localStorage
        let app_state = load_state().unwrap_or_else(|| AppState {
            completed_tasks: Vec::new(),
            task_history: Vec::new(),
            completed_pomodoros: 0,
        });

        Self {
            time: config.work_duration,
            running: false,
            interval: None,
            current_task: String::new(),
            completed_tasks: app_state.completed_tasks,
            state: TimerState::Working,
            completed_pomodoros: app_state.completed_pomodoros,
            markdown_visible: false,
            markdown_content: String::new(),
            config,
            task_history: app_state.task_history,
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
            Msg::Reset => {
                self.time = match self.state {
                    TimerState::Working => self.config.work_duration,
                    TimerState::ShortBreak => self.config.short_break,
                    TimerState::LongBreak => self.config.long_break,
                };
                self.running = false;
                self.interval = None;
                true
            }
            Msg::Tick => {
                if self.time > 0 {
                    self.time -= 1;
                    if self.time == 0 {
                        ctx.link().send_message(Msg::TimerComplete);
                    }
                    true
                } else {
                    self.running = false;
                    self.interval = None;
                    true
                }
            }
            Msg::TimerComplete => {
                match self.state {
                    TimerState::Working => {
                        if !self.current_task.is_empty() {
                            self.completed_pomodoros += 1;
                            ctx.link().send_message(Msg::CompleteTask);
                            if self.completed_pomodoros % self.config.pomodoros_until_long_break
                                == 0
                            {
                                self.state = TimerState::LongBreak;
                                self.time = self.config.long_break;
                            } else {
                                self.state = TimerState::ShortBreak;
                                self.time = self.config.short_break;
                            }
                            self.save_app_state();
                        }
                    }
                    TimerState::ShortBreak | TimerState::LongBreak => {
                        self.state = TimerState::Working;
                        self.time = self.config.work_duration;
                    }
                }
                true
            }
            Msg::UpdateTask(task) => {
                self.current_task = task;
                true
            }
            Msg::CompleteTask => {
                if !self.current_task.is_empty() {
                    let actual_duration = self.config.work_duration - self.time;
                    let task = Task::new(self.current_task.clone(), actual_duration);

                    if !self.task_history.contains(&self.current_task) {
                        self.task_history.push(self.current_task.clone());
                    }

                    self.completed_tasks.push(task);
                    self.current_task.clear();

                    self.running = false;
                    self.interval = None;
                    self.time = self.config.work_duration;

                    self.save_app_state();
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
            Msg::SelectPreviousTask(task) => {
                if !task.is_empty() {
                    self.current_task = task;
                }
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
        let state_text = match self.state {
            TimerState::Working => "作業時間",
            TimerState::ShortBreak => "小休憩",
            TimerState::LongBreak => "長休憩",
        };

        html! {
            <div class="p-4 max-w-2xl mx-auto">
                <h1 class="text-2xl font-bold mb-4 text-center">{"ポモドーロタイマー"}</h1>
                <div class="space-y-6">
                    <div class="text-center">
                        <p class="text-lg font-bold mb-2">{state_text}</p>
                        <p class="text-5xl font-mono mb-4">{format!("{:02}:{:02}", minutes, seconds)}</p>
                        <p class="text-sm text-gray-600 mb-4">
                            {format!("完了したポモドーロ: {}", self.completed_pomodoros)}
                        </p>
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
                            if !self.task_history.is_empty() {
                                <select
                                    class="w-full px-3 py-2 border rounded focus:outline-none focus:border-blue-500 mb-2"
                                    onchange={ctx.link().callback(|e: Event| {
                                        let select = e.target_unchecked_into::<HtmlSelectElement>();
                                        Msg::SelectPreviousTask(select.value())
                                    })}
                                >
                                    <option value="">{"過去のタスクから選択..."}</option>
                                    {for self.task_history.iter().map(|task| {
                                        html! {
                                            <option value={task.clone()}>{task}</option>
                                        }
                                    })}
                                </select>
                            }

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
                                let completed_at_local = task.completed_at.with_timezone(&chrono::Local);
                                html! {
                                    <div class="p-2 bg-gray-100 rounded">
                                        <p class="text-sm">{&task.description}</p>
                                        <p class="text-xs text-gray-500">
                                            {format!("作業時間: {}分 ({})",
                                                duration_minutes,
                                                completed_at_local.format("%Y-%m-%d %H:%M:%S"))}
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
