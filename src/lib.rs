pub mod battery;
pub mod config;
pub mod controller;
pub mod dialog;
pub mod notifier;
pub mod rumble;
pub mod single_instance;
pub mod startup;
pub mod toast;

pub type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
