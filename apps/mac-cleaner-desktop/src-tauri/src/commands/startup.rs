use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::MessageDialogKind;

use super::helpers::{confirm_native, display_error};

#[tauri::command]
pub(crate) fn launch_at_login(app: AppHandle) -> Result<LoginItemStatus, String> {
    let available = installed_app_bundle().is_some();
    let enabled = if available {
        app.autolaunch().is_enabled().map_err(display_error)?
    } else {
        false
    };
    Ok(LoginItemStatus { available, enabled })
}

#[tauri::command]
pub(crate) async fn set_launch_at_login(
    app: AppHandle,
    enabled: bool,
) -> Result<LoginItemStatus, String> {
    if installed_app_bundle().is_none() {
        return Err(
            "Launch at login is only available from an installed Mac Cleaner.app bundle."
                .to_owned(),
        );
    }
    let action = if enabled { "enable" } else { "disable" };
    let prompt =
        format!("Do you want to {action} Launch at login for this installed Mac Cleaner app?");
    let confirmed = confirm_native(
        &app,
        "Confirm startup setting",
        prompt,
        MessageDialogKind::Info,
        format!("{action} Launch at login"),
    )
    .await?;
    if !confirmed {
        return Err("Startup setting change cancelled. Nothing was changed.".to_owned());
    }
    if enabled {
        app.autolaunch().enable().map_err(display_error)?;
    } else {
        app.autolaunch().disable().map_err(display_error)?;
    }
    Ok(LoginItemStatus {
        available: true,
        enabled: app.autolaunch().is_enabled().map_err(display_error)?,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LoginItemStatus {
    available: bool,
    enabled: bool,
}

fn installed_app_bundle() -> Option<std::path::PathBuf> {
    let executable = std::env::current_exe().ok()?.canonicalize().ok()?;
    executable.ancestors().find_map(|ancestor| {
        (ancestor
            .extension()
            .is_some_and(|extension| extension == "app"))
        .then(|| ancestor.to_path_buf())
    })
}

#[cfg(test)]
mod tests {
    use super::installed_app_bundle;

    #[test]
    fn development_binary_cannot_be_registered_as_a_login_item() {
        assert!(installed_app_bundle().is_none());
    }
}
