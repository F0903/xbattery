pub mod audio;
pub mod config;
pub mod console;
pub mod controller;
pub mod dialog;
pub mod elevate;
pub mod ipc;
pub mod launch_context;
pub mod notifier;
pub mod single_instance;
pub mod startup;
pub mod toast;
pub mod update;

pub type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
