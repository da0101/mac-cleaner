use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

pub(super) fn display_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

pub(super) async fn confirm_native(
    app: &AppHandle,
    title: impl Into<String>,
    message: impl Into<String>,
    kind: MessageDialogKind,
    confirm_label: impl Into<String>,
) -> Result<bool, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Mac Cleaner window is unavailable. Nothing was changed.".to_owned())?;
    window.show().map_err(display_error)?;
    window.unminimize().map_err(display_error)?;
    window.set_focus().map_err(display_error)?;

    let (sender, receiver) = tokio::sync::oneshot::channel();
    app.dialog()
        .message(message)
        .title(title)
        .kind(kind)
        .buttons(MessageDialogButtons::OkCancelCustom(
            confirm_label.into(),
            "Cancel".to_owned(),
        ))
        .parent(&window)
        .show(move |confirmed| {
            let _ = sender.send(confirmed);
        });

    receiver
        .await
        .map_err(|_| "Native confirmation closed unexpectedly. Nothing was changed.".to_owned())
}
