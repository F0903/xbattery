mod app;
pub mod audio;
mod cli;
mod config;
mod console;
mod controller;
mod dialog;
mod elevate;
mod ipc;
mod launch_context;
mod notifier;
mod single_instance;
mod startup;
mod update;

pub use app::run;

pub type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
