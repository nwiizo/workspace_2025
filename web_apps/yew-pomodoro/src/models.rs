use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub description: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub completed_at: DateTime<Utc>,
    pub duration: i32,
    pub pomodoro_count: i32,
}

impl Task {
    pub fn new(description: String, duration: i32) -> Self {
        Self {
            description,
            completed_at: Utc::now(),
            duration,
            pomodoro_count: 1,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimerConfig {
    pub work_duration: i32,
    pub short_break: i32,
    pub long_break: i32,
    pub pomodoros_until_long_break: i32,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            work_duration: 25 * 60,
            short_break: 5 * 60,
            long_break: 15 * 60,
            pomodoros_until_long_break: 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TimerState {
    Working,
    ShortBreak,
    LongBreak,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppState {
    pub completed_tasks: Vec<Task>,
    pub task_history: Vec<String>,
    pub completed_pomodoros: i32,
}
