//! Auto-clean authorization: which apps may be asked to quit, and when.
//!
//! The policy is pure. It reads a memory snapshot, a process list, and the
//! owner's opt-in lists, and returns a decision. It never signals a process;
//! the native app asks the chosen app to quit through the normal macOS quit
//! request, so documents can still prompt to save.

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Duration, Utc};
use cleaner_domain::{
    AppEnergy, AutoCleanSettings, AutoQuitTarget, AutoRamDecision, AutoRamSettings, AutoRamState,
    MemorySnapshot, ProcessSnapshot,
};

use crate::{PolicyError, catalog};

/// Apps that can never be on the quit list, whatever the owner types.
const NEVER_QUIT: &[&str] = &[
    "finder",
    "dock",
    "loginwindow",
    "systemuiserver",
    "windowserver",
    "mac cleaner",
    "docker",
    "activity monitor",
    "system settings",
    "keychain",
    "passwords",
    "1password",
    "time machine",
    "simulator",
    "android studio",
    "claude",
    "codex",
    "terminal",
    "iterm",
    "xcode",
    "visual studio code",
    "cursor",
    "intellij",
    "pycharm",
    "webstorm",
    "goland",
    "clion",
    "datagrip",
    "rubymine",
    "phpstorm",
    "sublime",
    "windsurf",
    "orbstack",
    "parallels",
    "vmware",
    "virtualbox",
];

/// Names too short to match as a substring (`Code` would also hit `Barcode`).
const NEVER_QUIT_EXACT: &[&str] = &["code", "zed", "nova", "rider", "utm"];

/// A request that was quit within this window and is still running counts as refused.
const RETRY_WINDOW_MINUTES: i64 = 30;
const MAX_ATTEMPTS_IN_WINDOW: usize = 2;
const MAX_APP_NAME_LEN: usize = 80;
const MAX_QUIT_APPS: usize = 40;

#[must_use]
pub fn is_never_quit(name: &str) -> bool {
    let lower = name.trim().to_lowercase();
    NEVER_QUIT_EXACT.contains(&lower.as_str())
        || NEVER_QUIT.iter().any(|blocked| lower.contains(blocked))
}

/// Clamp and clean owner-supplied settings. Unknown or protected entries are
/// rejected rather than silently kept, so the owner sees what was refused.
pub fn validate_auto_settings(
    mut settings: AutoCleanSettings,
) -> Result<AutoCleanSettings, PolicyError> {
    let gib = cleaner_domain::GIB;
    settings.ram.min_available_bytes = settings.ram.min_available_bytes.clamp(gib / 2, 8 * gib);
    settings.ram.sustained_secs = settings.ram.sustained_secs.clamp(30, 3_600);
    settings.ram.cooldown_secs = settings.ram.cooldown_secs.clamp(60, 3_600);

    let mut apps: Vec<String> = Vec::new();
    for raw in &settings.ram.quit_apps {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        if name.len() > MAX_APP_NAME_LEN || name.contains('/') || name.contains('\0') {
            return Err(PolicyError::InvalidAutoSettings(format!(
                "`{name}` is not a valid app name"
            )));
        }
        if is_never_quit(name) {
            return Err(PolicyError::InvalidAutoSettings(format!(
                "{name} is protected and can never be quit automatically"
            )));
        }
        if !apps.iter().any(|known| known.eq_ignore_ascii_case(name)) {
            apps.push(name.to_owned());
        }
    }
    if apps.len() > MAX_QUIT_APPS {
        return Err(PolicyError::InvalidAutoSettings(format!(
            "at most {MAX_QUIT_APPS} apps can be listed"
        )));
    }
    settings.ram.quit_apps = apps;

    settings.storage.interval_hours = settings.storage.interval_hours.clamp(1, 168);
    settings.storage.min_idle_hours = settings.storage.min_idle_hours.clamp(24, 24 * 90);
    settings.storage.low_free_bytes = settings.storage.low_free_bytes.min(500 * gib);
    let known: Vec<&str> = catalog().iter().map(|target| target.id).collect();
    let mut ids: Vec<String> = Vec::new();
    for id in &settings.storage.target_ids {
        if !known.contains(&id.as_str()) {
            return Err(PolicyError::InvalidAutoSettings(format!(
                "`{id}` is not a cleanup category"
            )));
        }
        if !ids.contains(id) {
            ids.push(id.clone());
        }
    }
    settings.storage.target_ids = ids;
    Ok(settings)
}

/// `/Applications/Google Chrome.app/Contents/…` → (`Google Chrome`, `/Applications/Google Chrome.app`).
///
/// Only bundles inside an `Applications` folder qualify. The system's own
/// bundles are never returned.
#[must_use]
pub fn app_bundle(executable: &str) -> Option<(String, String)> {
    let path = Path::new(executable);
    if executable.starts_with("/System/") || executable.starts_with("/Library/") {
        return None;
    }
    let mut built = std::path::PathBuf::new();
    let mut parent_is_applications = false;
    for component in path.components() {
        let part = component.as_os_str().to_string_lossy();
        built.push(component);
        if let Some(name) = part.strip_suffix(".app") {
            if !parent_is_applications || name.is_empty() {
                return None;
            }
            return Some((name.to_owned(), built.to_string_lossy().into_owned()));
        }
        parent_is_applications = part == "Applications";
    }
    None
}

/// Decide whether to ask an opted-in app to quit right now.
///
/// Mutates `state` only for the "memory has been low since" clock and to
/// forget old quit requests; recording a request is the caller's job once the
/// quit was actually asked for.
pub fn plan_ram_relief(
    settings: &AutoRamSettings,
    memory: &MemorySnapshot,
    processes: &[ProcessSnapshot],
    state: &mut AutoRamState,
    now: DateTime<Utc>,
) -> AutoRamDecision {
    if !settings.enabled {
        state.low_since = None;
        return AutoRamDecision::Disabled;
    }
    state
        .recent_quits
        .retain(|(_, at)| now - *at < Duration::minutes(RETRY_WINDOW_MINUTES));

    if memory.available_bytes >= settings.min_available_bytes {
        state.low_since = None;
        return AutoRamDecision::Healthy;
    }
    let low_since = *state.low_since.get_or_insert(now);
    let low_for = secs_between(low_since, now);
    if low_for < settings.sustained_secs {
        return AutoRamDecision::Waiting {
            secs_remaining: settings.sustained_secs - low_for,
        };
    }
    if let Some(last) = state.last_quit_at {
        let since = secs_between(last, now);
        if since < settings.cooldown_secs {
            return AutoRamDecision::Cooldown {
                secs_remaining: settings.cooldown_secs - since,
            };
        }
    }
    if settings.quit_apps.is_empty() {
        return AutoRamDecision::NoTargets {
            reason: "No apps are on your quit list".to_owned(),
        };
    }

    let mut apps: HashMap<String, Group> = HashMap::new();
    for process in processes {
        let Some((name, path)) = process.executable.as_deref().and_then(app_bundle) else {
            continue;
        };
        let group = apps.entry(path.clone()).or_insert_with(|| Group {
            name,
            path,
            bytes: 0,
            pids: Vec::new(),
            protected: false,
        });
        group.bytes = group.bytes.saturating_add(process.memory_bytes);
        group.pids.push(process.pid);
        group.protected |= process.protected;
    }

    let mut candidates: Vec<Group> = apps
        .into_values()
        .filter(|group| {
            settings
                .quit_apps
                .iter()
                .any(|listed| listed.eq_ignore_ascii_case(&group.name))
        })
        .collect();
    if candidates.is_empty() {
        return AutoRamDecision::NoTargets {
            reason: "None of the apps on your quit list is running".to_owned(),
        };
    }
    candidates.retain(|group| !group.protected && !is_never_quit(&group.name));
    if candidates.is_empty() {
        return AutoRamDecision::NoTargets {
            reason: "The listed apps are protected right now".to_owned(),
        };
    }
    candidates.retain(|group| {
        state
            .recent_quits
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case(&group.name))
            .count()
            < MAX_ATTEMPTS_IN_WINDOW
    });
    let Some(best) = candidates.into_iter().max_by_key(|group| group.bytes) else {
        return AutoRamDecision::NoTargets {
            reason: "The listed apps did not quit when asked".to_owned(),
        };
    };
    let mut pids = best.pids;
    pids.sort_unstable();
    AutoRamDecision::Quit {
        target: AutoQuitTarget {
            app_name: best.name,
            app_path: best.path,
            bytes: best.bytes,
            pids,
        },
    }
}

struct Group {
    name: String,
    path: String,
    bytes: u64,
    pids: Vec<u32>,
    protected: bool,
}

fn secs_between(earlier: DateTime<Utc>, later: DateTime<Utc>) -> u64 {
    u64::try_from((later - earlier).num_seconds().max(0)).unwrap_or(0)
}

/// Substring names for the app picker, so the screen never offers an app Rust would refuse.
#[must_use]
pub fn never_quit_names() -> Vec<String> {
    NEVER_QUIT
        .iter()
        .chain(NEVER_QUIT_EXACT.iter())
        .map(|name| (*name).to_owned())
        .collect()
}

/// Add every process into its app, so VS Code or Chrome appear once with all
/// their helpers. Heaviest CPU first, then memory. Pure: it never signals anything.
#[must_use]
pub fn group_app_energy(processes: &[ProcessSnapshot], home: &str) -> Vec<AppEnergy> {
    let user_apps = format!("{home}/Applications/");
    let mut groups: HashMap<String, AppEnergy> = HashMap::new();
    for process in processes {
        let bundle = process.executable.as_deref().and_then(app_bundle);
        let (key, name, app_path) = match &bundle {
            Some((name, path)) => (path.clone(), name.clone(), Some(path.clone())),
            None => (format!("proc:{}", process.name), process.name.clone(), None),
        };
        let group = groups.entry(key).or_insert_with(|| AppEnergy {
            name,
            app_path,
            process_count: 0,
            memory_bytes: 0,
            cpu_percent: 0.0,
            protected: false,
            protection_reason: None,
            can_quit: false,
        });
        group.process_count += 1;
        group.memory_bytes = group.memory_bytes.saturating_add(process.memory_bytes);
        group.cpu_percent += f64::from(process.cpu_percent);
        if process.protected {
            group.protected = true;
            if group.protection_reason.is_none() {
                group
                    .protection_reason
                    .clone_from(&process.protection_reason);
            }
        }
    }
    let mut apps: Vec<AppEnergy> = groups.into_values().collect();
    for app in &mut apps {
        app.protected |= is_never_quit(&app.name);
        app.can_quit = !app.protected
            && app.app_path.as_deref().is_some_and(|path| {
                path.starts_with("/Applications/") || path.starts_with(&user_apps)
            });
    }
    apps.sort_by(|a, b| {
        b.cpu_percent
            .total_cmp(&a.cpu_percent)
            .then(b.memory_bytes.cmp(&a.memory_bytes))
    });
    apps
}
