use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json, Redirect},
    routing::get,
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    env,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::net::TcpListener;
use tracing::info;
use uuid::Uuid;

const SECONDS_PER_USER: u64 = 30;
const MAX_CONCURRENT_USERS: usize = 5;

#[derive(Clone)]
struct AppState {
    queue: Arc<Mutex<WaitingQueue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueueEntry {
    user_id: String,
    joined_at: u64,
}

struct WaitingQueue {
    entries: VecDeque<QueueEntry>,
    active_users: Vec<String>,
    start_time: Instant,
}

impl WaitingQueue {
    fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            active_users: Vec::new(),
            start_time: Instant::now(),
        }
    }

    fn current_time_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    fn total_in_queue(&self) -> usize {
        self.entries.len()
    }

    fn active_count(&self) -> usize {
        self.active_users.len()
    }

    fn join(&mut self, user_id: &str) -> usize {
        if let Some(pos) = self.entries.iter().position(|e| e.user_id == user_id) {
            return pos + 1;
        }

        if self.active_users.contains(&user_id.to_string()) {
            return 0;
        }

        self.entries.push_back(QueueEntry {
            user_id: user_id.to_string(),
            joined_at: self.current_time_secs(),
        });

        self.entries.len()
    }

    fn can_enter(&mut self, user_id: &str) -> bool {
        if self.active_users.contains(&user_id.to_string()) {
            return true;
        }

        if self.active_users.len() >= MAX_CONCURRENT_USERS {
            return false;
        }

        if let Some(front) = self.entries.front() {
            if front.user_id == user_id {
                self.entries.pop_front();
                self.active_users.push(user_id.to_string());
                return true;
            }
        }

        false
    }

    fn estimated_wait(&self, position: usize) -> Duration {
        let users_ahead = position.saturating_sub(1);
        Duration::from_secs((users_ahead as u64) * SECONDS_PER_USER)
    }
}

#[derive(Serialize)]
struct QueueStatus {
    position: usize,
    total_waiting: usize,
    active_users: usize,
    max_concurrent: usize,
    estimated_wait_seconds: u64,
    can_enter: bool,
    redirect_url: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        queue: Arc::new(Mutex::new(WaitingQueue::new())),
    };

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/", get(handle_waiting_room))
        .route("/waiting-room", get(handle_waiting_room))
        .route("/api/status", get(handle_status))
        .route("/health", get(handle_health))
        .with_state(state);

    info!("Waiting Room App is running on port {}", port);

    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn handle_status(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let user_id = jar
        .get("waiting_room_id")
        .map(|c| c.value().to_string())
        .unwrap_or_default();

    if user_id.is_empty() {
        return Json(QueueStatus {
            position: 0,
            total_waiting: 0,
            active_users: 0,
            max_concurrent: MAX_CONCURRENT_USERS,
            estimated_wait_seconds: 0,
            can_enter: false,
            redirect_url: None,
        });
    }

    let (position, total, active, can_enter, wait) = {
        let mut queue = state.queue.lock().unwrap();
        let pos = queue.join(&user_id);
        let can = queue.can_enter(&user_id);
        let total = queue.total_in_queue();
        let active = queue.active_count();
        let wait = queue.estimated_wait(pos);
        (pos, total, active, can, wait)
    };

    let redirect_url = if can_enter {
        Some(env::var("MAIN_APP_URL").unwrap_or_else(|_| "/shop".to_string()))
    } else {
        None
    };

    Json(QueueStatus {
        position,
        total_waiting: total,
        active_users: active,
        max_concurrent: MAX_CONCURRENT_USERS,
        estimated_wait_seconds: wait.as_secs(),
        can_enter,
        redirect_url,
    })
}

async fn handle_waiting_room(
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let user_id = jar
        .get("waiting_room_id")
        .map(|c| c.value().to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let (position, can_enter, estimated_wait, total, active) = {
        let mut queue = state.queue.lock().unwrap();
        let pos = queue.join(&user_id);
        let can_enter = queue.can_enter(&user_id);
        let wait = queue.estimated_wait(pos);
        let total = queue.total_in_queue();
        let active = queue.active_count();
        (pos, can_enter, wait, total, active)
    };

    if can_enter {
        let redirect_url = env::var("MAIN_APP_URL").unwrap_or_else(|_| "/shop".to_string());
        return (
            jar.add(Cookie::new("waiting_room_id", user_id)),
            Redirect::temporary(&redirect_url),
        )
            .into_response();
    }

    let wait_seconds = estimated_wait.as_secs();

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>待合室 - Limited Edition Shop</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: white;
        }}
        .container {{
            text-align: center;
            padding: 40px;
            max-width: 600px;
            width: 100%;
        }}
        .logo {{
            font-size: 1.2em;
            color: #888;
            margin-bottom: 10px;
        }}
        h1 {{
            font-size: 2.2em;
            margin-bottom: 30px;
            color: #e94560;
        }}
        .queue-card {{
            background: rgba(255,255,255,0.08);
            border-radius: 20px;
            padding: 40px;
            margin-bottom: 30px;
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255,255,255,0.1);
        }}
        .position-wrapper {{
            margin-bottom: 30px;
        }}
        .position {{
            font-size: 5em;
            font-weight: bold;
            background: linear-gradient(135deg, #e94560, #ff6b6b);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            line-height: 1;
        }}
        .position-label {{
            font-size: 1.1em;
            color: #aaa;
            margin-top: 10px;
        }}
        .stats {{
            display: flex;
            justify-content: center;
            gap: 40px;
            margin: 30px 0;
            flex-wrap: wrap;
        }}
        .stat {{
            text-align: center;
        }}
        .stat-value {{
            font-size: 2em;
            font-weight: bold;
            color: #e94560;
        }}
        .stat-label {{
            font-size: 0.9em;
            color: #888;
            margin-top: 5px;
        }}
        .countdown {{
            background: rgba(233, 69, 96, 0.2);
            border-radius: 15px;
            padding: 20px;
            margin: 20px 0;
        }}
        .countdown-time {{
            font-size: 2.5em;
            font-weight: bold;
            font-family: monospace;
            color: #e94560;
        }}
        .countdown-label {{
            font-size: 0.9em;
            color: #aaa;
            margin-top: 5px;
        }}
        .progress-container {{
            margin-top: 30px;
        }}
        .progress-bar {{
            width: 100%;
            height: 10px;
            background: rgba(255,255,255,0.1);
            border-radius: 5px;
            overflow: hidden;
        }}
        .progress {{
            height: 100%;
            background: linear-gradient(90deg, #e94560, #ff6b6b);
            border-radius: 5px;
            transition: width 1s ease;
        }}
        .progress-text {{
            font-size: 0.85em;
            color: #888;
            margin-top: 10px;
        }}
        .note {{
            margin-top: 30px;
            color: #666;
            font-size: 0.9em;
            line-height: 1.6;
        }}
        .status-indicator {{
            display: inline-block;
            width: 10px;
            height: 10px;
            border-radius: 50%;
            background: #4CAF50;
            margin-right: 8px;
            animation: blink 1.5s ease-in-out infinite;
        }}
        @keyframes blink {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.5; }}
        }}
        .redirect-notice {{
            display: none;
            background: linear-gradient(135deg, #4CAF50, #45a049);
            border-radius: 15px;
            padding: 30px;
            margin-top: 20px;
            animation: fadeIn 0.5s ease;
        }}
        .redirect-notice.show {{
            display: block;
        }}
        .redirect-notice h2 {{
            color: white;
            margin-bottom: 10px;
        }}
        @keyframes fadeIn {{
            from {{ opacity: 0; transform: translateY(-10px); }}
            to {{ opacity: 1; transform: translateY(0); }}
        }}
        @media (max-width: 480px) {{
            .container {{ padding: 20px; }}
            .queue-card {{ padding: 25px; }}
            .position {{ font-size: 3.5em; }}
            .stats {{ gap: 20px; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">Limited Edition Shop</div>
        <h1>待合室</h1>

        <div class="queue-card">
            <div class="position-wrapper">
                <div class="position" id="position">{}</div>
                <div class="position-label">あなたの順番</div>
            </div>

            <div class="stats">
                <div class="stat">
                    <div class="stat-value" id="total-waiting">{}</div>
                    <div class="stat-label">待機中</div>
                </div>
                <div class="stat">
                    <div class="stat-value" id="active-users">{}/{}</div>
                    <div class="stat-label">ショップ内</div>
                </div>
            </div>

            <div class="countdown">
                <div class="countdown-time" id="countdown">--:--</div>
                <div class="countdown-label">推定待ち時間</div>
            </div>

            <div class="progress-container">
                <div class="progress-bar">
                    <div class="progress" id="progress" style="width: 0%;"></div>
                </div>
                <div class="progress-text">
                    <span class="status-indicator"></span>
                    <span id="status-text">接続中...</span>
                </div>
            </div>
        </div>

        <div class="redirect-notice" id="redirect-notice">
            <h2>お待たせしました！</h2>
            <p>ショップページへ移動します...</p>
        </div>

        <p class="note">
            ページを閉じないでください。<br>
            順番が来ると自動的にショップへ移動します。
        </p>
    </div>

    <script>
        let estimatedWait = {};
        let startTime = Date.now();
        let checkInterval;

        function formatTime(seconds) {{
            if (seconds <= 0) return "まもなく";
            const mins = Math.floor(seconds / 60);
            const secs = seconds % 60;
            return `${{String(mins).padStart(2, '0')}}:${{String(secs).padStart(2, '0')}}`;
        }}

        function updateCountdown() {{
            const elapsed = Math.floor((Date.now() - startTime) / 1000);
            const remaining = Math.max(0, estimatedWait - elapsed);
            document.getElementById('countdown').textContent = formatTime(remaining);

            // Update progress
            if (estimatedWait > 0) {{
                const progress = Math.min(100, (elapsed / estimatedWait) * 100);
                document.getElementById('progress').style.width = progress + '%';
            }}
        }}

        async function checkStatus() {{
            try {{
                const response = await fetch('/api/status');
                const data = await response.json();

                document.getElementById('position').textContent = data.position;
                document.getElementById('total-waiting').textContent = data.total_waiting;
                document.getElementById('active-users').textContent =
                    data.active_users + '/' + data.max_concurrent;
                document.getElementById('status-text').textContent = 'リアルタイム更新中';

                if (data.can_enter && data.redirect_url) {{
                    document.getElementById('redirect-notice').classList.add('show');
                    clearInterval(checkInterval);
                    setTimeout(() => {{
                        window.location.href = data.redirect_url;
                    }}, 1500);
                    return;
                }}

                // Update estimated wait if changed significantly
                if (Math.abs(data.estimated_wait_seconds - estimatedWait) > 10) {{
                    estimatedWait = data.estimated_wait_seconds;
                    startTime = Date.now();
                }}
            }} catch (e) {{
                document.getElementById('status-text').textContent = '再接続中...';
            }}
        }}

        // Initial check
        checkStatus();

        // Poll every 3 seconds
        checkInterval = setInterval(checkStatus, 3000);

        // Update countdown every second
        setInterval(updateCountdown, 1000);
    </script>
</body>
</html>"#,
        position, total, active, MAX_CONCURRENT_USERS, wait_seconds
    );

    (
        jar.add(Cookie::new("waiting_room_id", user_id)),
        Html(html),
    )
        .into_response()
}

async fn handle_health() -> impl IntoResponse {
    (StatusCode::OK, "Healthy")
}
