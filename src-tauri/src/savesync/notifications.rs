use crate::{app_handle, app_store};
use tauri::Runtime;
use tauri_plugin_notification::{NotificationBuilder, NotificationExt};

pub struct DesktopNotification {
    pub title: Option<String>,
    pub body: Option<String>,
    pub silent: bool,
}

fn run_func<R, F, T>(
    builder: NotificationBuilder<R>,
    operation: F,
    value: Option<T>,
) -> NotificationBuilder<R>
where
    R: Runtime,
    F: Fn(NotificationBuilder<R>, T) -> NotificationBuilder<R>,
{
    if let Some(value) = value {
        operation(builder, value)
    } else {
        builder
    }
}

pub fn desktop_notify(
    DesktopNotification {
        title,
        body,
        silent,
    }: DesktopNotification,
) {
    let mut builder = app_handle().notification().builder();

    builder = run_func(builder, NotificationBuilder::title, title);

    builder = run_func(builder, NotificationBuilder::body, body);

    builder = if silent {
        builder.silent()
    } else {
        builder.sound("Default")
    };

    builder.show().unwrap();
}

pub fn sync_notify(notif_struct: DesktopNotification) {
    if app_store().sync_notifications() {
        desktop_notify(notif_struct);
    }
}
