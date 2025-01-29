mod models;
mod storage;
mod timer;

use timer::PomodoroTimer;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<PomodoroTimer>::new().render();
}
