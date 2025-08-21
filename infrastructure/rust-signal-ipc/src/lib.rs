/// rust-signal-ipc ライブラリのルートモジュール
///
/// このライブラリは、Rustでシグナル処理とプロセス間通信を
/// 実装するための基本的な機能を提供します。

pub mod errors;
pub mod ipc;

pub use errors::{IPCError, Result};