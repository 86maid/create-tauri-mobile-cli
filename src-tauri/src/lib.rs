use clap::Parser;
use std::{
    io::Read,
    sync::{Arc, Mutex, OnceLock},
};
use tauri::{async_runtime::JoinHandle, AppHandle, Emitter, Event, EventId, Listener, Manager};
use tokio::sync::Notify;

#[tauri::command]
async fn main() {
    hook_stdout().await;

    // your code

    unhook_stdout();
}

pub fn listen(callback: impl Fn(Event) + Send + 'static) {
    APP_HANDLE.get().unwrap().listen("cli-read", callback);
}

pub fn unlisten(id: EventId) {
    APP_HANDLE.get().unwrap().unlisten(id);
}

pub fn listen_once(callback: impl FnOnce(Event) + Send + 'static) {
    APP_HANDLE.get().unwrap().once("cli-read", callback);
}

pub fn emit(text: impl AsRef<str>) {
    APP_HANDLE
        .get()
        .unwrap()
        .emit("cli-write", text.as_ref())
        .unwrap();
}

pub fn emitln(text: impl AsRef<str>) {
    APP_HANDLE
        .get()
        .unwrap()
        .emit("cli-write", format!("{}\n", text.as_ref()))
        .unwrap();
}

pub fn clear() {
    APP_HANDLE.get().unwrap().emit("cli-cmd", "clear").unwrap();
}

pub fn clap_parse<T: Parser>(callback: impl Fn(T) + Send + 'static) {
    listen(move |e| {
        let args = e
            .payload()
            .strip_prefix('"')
            .unwrap()
            .strip_suffix('"')
            .unwrap()
            .split_whitespace()
            .map(|v| v.to_string());

        match T::try_parse_from(args) {
            Ok(v) => {
                APP_HANDLE.get().unwrap().unlisten(e.id());
                callback(v)
            }
            Err(v) => emitln(format!("{}", v)),
        }
    });
}

pub async fn hook_stdout() {
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    let handle = tauri::async_runtime::spawn_blocking(move || {
        let mut shh = shh::stdout().unwrap();
        notify_clone.notify_one();

        loop {
            let mut buf = Vec::new();
            if shh.read_to_end(&mut buf).unwrap() != 0 {
                emit(String::from_utf8_lossy(&buf));
            }
        }
    });

    notify.notified().await;

    *STDOUT_TASK.get_or_init(Default::default).lock().unwrap() = Some(handle);
}

pub fn unhook_stdout() {
    if let Some(handle) = STDOUT_TASK
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .take()
    {
        handle.abort();
    }
}

pub async fn hook_stderr() {
    let notify = std::sync::Arc::new(Notify::new());
    let notify_clone = notify.clone();

    let handle = tauri::async_runtime::spawn_blocking(move || {
        let mut shh = shh::stderr().unwrap();
        notify_clone.notify_one();

        loop {
            let mut buf = Vec::new();
            if shh.read_to_end(&mut buf).unwrap() != 0 {
                emit(String::from_utf8_lossy(&buf));
            }
        }
    });

    notify.notified().await;

    *STDERR_TASK.get_or_init(Default::default).lock().unwrap() = Some(handle);
}

pub fn unhook_stderr() {
    if let Some(handle) = STDERR_TASK
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .take()
    {
        handle.abort();
    }
}

static STDOUT_TASK: OnceLock<Mutex<Option<JoinHandle<()>>>> = OnceLock::new();
static STDERR_TASK: OnceLock<Mutex<Option<JoinHandle<()>>>> = OnceLock::new();
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![main])
        .setup(|app| {
            APP_HANDLE.set(app.app_handle().clone()).unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
