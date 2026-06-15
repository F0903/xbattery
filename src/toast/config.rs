const DEFAULT_APP_ID: &str = "xbattery";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToastConfig {
    app_id: String,
}

impl ToastConfig {
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
        }
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }
}

impl Default for ToastConfig {
    fn default() -> Self {
        Self::new(DEFAULT_APP_ID)
    }
}
