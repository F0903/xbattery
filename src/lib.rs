pub mod battery;
pub mod config;
pub mod controller;
pub mod gameinput;
pub mod notifier;
pub mod toast;
pub mod winrt_input;
pub mod xinput;

pub type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
