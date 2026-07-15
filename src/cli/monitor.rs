use crate::{
    AppResult,
    config::{AppConfig, ConfigIssue, ConfigWatchEvent},
    controller::service::ControllerService,
    ipc::{BACKGROUND_INSTANCE_MUTEX_NAME, BACKGROUND_INSTANCE_STOP_EVENT_NAME, NamedEvent},
    notifier::{Notification, NotificationUrgency, Notifier, ToastNotifier},
    single_instance::SingleInstance,
    update,
};

pub(super) fn run() -> AppResult<()> {
    let Some(_guard) = SingleInstance::acquire(BACKGROUND_INSTANCE_MUTEX_NAME)? else {
        println!("xbattery monitor is already running.");
        return Ok(());
    };

    let stop_event = NamedEvent::open_or_create(BACKGROUND_INSTANCE_STOP_EVENT_NAME)?;

    let (loaded_config, config_watcher) = AppConfig::load_for_monitor()?;
    let config_path = loaded_config.path.clone();
    let mut config = loaded_config.config;
    let mut startup_issue = loaded_config.issue;
    let service_config = match config.controller_service_config() {
        Ok(service_config) => service_config,
        Err(error) => {
            let Some(path) = config_path.clone() else {
                return Err(error);
            };

            startup_issue = Some(ConfigIssue::new(
                path,
                format!("failed to prepare configuration: {error}"),
            ));
            config = AppConfig::default();
            config.controller_service_config()?
        }
    };

    let notifier = ToastNotifier::new(config.notifications.app_id.clone());
    let reload_notifier = notifier.clone();
    let update_handle = update::start_background_checks(config.updates.clone(), notifier.clone())?;

    if let Some(issue) = &startup_issue {
        report_config_issue(&notifier, issue);
    }

    let mut service = ControllerService::new(notifier, service_config);
    service.run_until_ctrl_c_or_reconfigure(|| stop_event.is_signaled(), {
        let notifier = reload_notifier;
        let mut active_updates = config.updates.clone();
        move || {
            let mut latest = None;

            if let Some(config_watcher) = &config_watcher {
                while let Ok(event) = config_watcher.try_recv() {
                    let (path, config) = match event {
                        ConfigWatchEvent::Loaded { path, config } => (path, config),
                        ConfigWatchEvent::Rejected(issue) => {
                            report_config_issue(&notifier, &issue);
                            continue;
                        }
                    };

                    let service_config = match config.controller_service_config() {
                        Ok(service_config) => service_config,
                        Err(error) => {
                            report_config_issue(
                                &notifier,
                                &ConfigIssue::new(
                                    path,
                                    format!("failed to prepare configuration: {error}"),
                                ),
                            );
                            continue;
                        }
                    };

                    if let Err(error) = update_handle.update_config(config.updates.clone()) {
                        report_config_issue(
                            &notifier,
                            &ConfigIssue::new(
                                path,
                                format!("failed to apply update settings: {error}"),
                            ),
                        );
                        continue;
                    }

                    if let Err(error) = notifier.set_app_id(config.notifications.app_id.clone()) {
                        if let Err(rollback_error) =
                            update_handle.update_config(active_updates.clone())
                        {
                            eprintln!(
                                "failed to restore the previous update configuration: {rollback_error}"
                            );
                        }
                        report_config_issue(
                            &notifier,
                            &ConfigIssue::new(
                                path,
                                format!("failed to apply notification settings: {error}"),
                            ),
                        );
                        continue;
                    }

                    active_updates = config.updates.clone();
                    latest = Some(service_config);
                }
            }

            Ok(latest)
        }
    })
}

fn report_config_issue(notifier: &impl Notifier, issue: &ConfigIssue) {
    eprintln!(
        "configuration at {} could not be used: {}",
        issue.path().display(),
        issue.message()
    );

    let notification = Notification::with_urgency(
        "xbattery configuration error",
        format!(
            "Could not use {}. Built-in defaults or the previous valid settings remain active. Fix and save the file to retry.\n\n{}",
            issue.path().display(),
            issue.message()
        ),
        NotificationUrgency::High,
    );

    if let Err(error) = notifier.notify(&notification) {
        eprintln!("failed to show configuration error notification: {error}");
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Mutex};

    use crate::{
        AppResult,
        config::ConfigIssue,
        notifier::{Notification, NotificationUrgency, Notifier},
    };

    use super::report_config_issue;

    #[derive(Default)]
    struct RecordingNotifier {
        notifications: Mutex<Vec<Notification>>,
    }

    impl Notifier for RecordingNotifier {
        fn notify(&self, notification: &Notification) -> AppResult<()> {
            self.notifications
                .lock()
                .unwrap()
                .push(notification.clone());
            Ok(())
        }
    }

    struct FailingNotifier;

    impl Notifier for FailingNotifier {
        fn notify(&self, _notification: &Notification) -> AppResult<()> {
            Err("notifications are unavailable".into())
        }
    }

    #[test]
    fn config_issues_are_reported_with_the_path_and_reason() {
        let notifier = RecordingNotifier::default();
        let issue = ConfigIssue::new(PathBuf::from("invalid.toml"), "unknown field");

        report_config_issue(&notifier, &issue);

        let notifications = notifier.notifications.lock().unwrap();
        let notification = notifications.first().unwrap();
        assert_eq!(notification.urgency(), NotificationUrgency::High);
        assert!(notification.body().contains("invalid.toml"));
        assert!(notification.body().contains("unknown field"));
    }

    #[test]
    fn config_issue_notification_failures_are_nonfatal() {
        let issue = ConfigIssue::new(PathBuf::from("invalid.toml"), "unknown field");

        report_config_issue(&FailingNotifier, &issue);
    }
}
