use std::path::Path;

use crate::{catalog, catalog_root, is_protected, safe_auto_catalog};

#[test]
fn automatic_cleanup_catalog_starts_empty() {
    assert!(safe_auto_catalog().is_empty());
}

#[test]
fn catalog_never_points_at_a_protected_prefix() {
    let home = Path::new("/Users/tester");
    for target in catalog() {
        assert!(catalog_root(home, target).is_ok(), "{}", target.id);
    }
}

#[test]
fn github_desktop_state_is_protected_in_both_directions() {
    let home = Path::new("/Users/tester");
    let state = home.join("Library/Application Support/GitHub Desktop");
    assert!(is_protected(home, &state));
    assert!(is_protected(home, &state.join("IndexedDB/repositories")));
    assert!(is_protected(
        home,
        &home.join("Library/Application Support")
    ));
}

#[test]
fn source_and_toolchain_roots_are_protected() {
    let home = Path::new("/Users/tester");
    assert!(is_protected(home, &home.join("Documents/GitHub/app")));
    assert!(is_protected(home, &home.join(".pub-cache/hosted")));
    assert!(is_protected(home, &home.join(".gradle/caches")));
    assert!(is_protected(home, Path::new("/tmp/outside-home")));
}

#[test]
fn named_and_project_targets_reject_anything_but_their_exact_entry() -> Result<(), &'static str> {
    use crate::{TargetLayout, candidate_path};
    let home = Path::new("/Users/tester");
    let named = catalog()
        .iter()
        .find(|target| target.id == "npm-content-cache")
        .ok_or("npm target missing")?;
    assert!(candidate_path(home, named, Path::new("_cacache")).is_ok());
    assert!(candidate_path(home, named, Path::new("_npx")).is_err());
    assert!(candidate_path(home, named, Path::new("_cacache/index")).is_err());

    let project = catalog()
        .iter()
        .find(|target| target.id == "project-node-modules")
        .ok_or("node target missing")?;
    assert!(matches!(project.layout, TargetLayout::Project { .. }));
    // No marker file exists under this fake home, so even a well-formed path fails closed.
    assert!(candidate_path(home, project, Path::new("app/node_modules")).is_err());
    assert!(candidate_path(home, project, Path::new("node_modules")).is_err());
    assert!(candidate_path(home, project, Path::new("app/src")).is_err());
    assert!(candidate_path(home, project, Path::new("a/b/c/node_modules")).is_err());
    assert!(candidate_path(home, project, Path::new("app/../node_modules")).is_err());
    Ok(())
}

#[test]
fn only_project_targets_may_be_discovered_below_documents() {
    for target in catalog() {
        if target.home_relative_root.starts_with("Documents") {
            assert!(target.allows_internal_links(), "{}", target.id);
            assert!(
                matches!(target.layout, crate::TargetLayout::Project { .. }),
                "{}",
                target.id
            );
        }
    }
}

#[test]
fn nested_project_entries_need_their_marker_and_exact_shape()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::{candidate_path, marker_present};
    let home = tempfile::TempDir::new()?;
    let project = home.path().join("Documents/GitHub/app");
    std::fs::create_dir_all(project.join("ios/Runner.xcodeproj"))?;
    std::fs::create_dir_all(project.join("ios/build"))?;
    let target = catalog()
        .iter()
        .find(|target| target.id == "project-ios-build")
        .ok_or("ios build target missing")?;
    assert_eq!(target.entry_depth(), 2);
    assert!(target.require_git_ignored);
    assert!(marker_present(&project, "ios/*.xcodeproj"));
    assert!(!marker_present(&project, "ios/*.xcworkspace"));

    assert!(candidate_path(home.path(), target, std::path::Path::new("app/ios/build")).is_ok());
    // Wrong tail, wrong depth, missing marker, or traversal all fail closed.
    assert!(candidate_path(home.path(), target, std::path::Path::new("app/ios/src")).is_err());
    assert!(candidate_path(home.path(), target, std::path::Path::new("ios/build")).is_err());
    assert!(candidate_path(home.path(), target, std::path::Path::new("a/b/c/ios/build")).is_err());
    assert!(candidate_path(home.path(), target, std::path::Path::new("other/ios/build")).is_err());
    assert!(
        candidate_path(
            home.path(),
            target,
            std::path::Path::new("app/../ios/build")
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn xcode_symlinked_project_markers_do_not_count() -> Result<(), Box<dyn std::error::Error>> {
    use crate::marker_present;
    let dir = tempfile::TempDir::new()?;
    std::fs::create_dir_all(dir.path().join("ios"))?;
    std::os::unix::fs::symlink("/tmp", dir.path().join("ios/Fake.xcodeproj"))?;
    assert!(!marker_present(dir.path(), "ios/*.xcodeproj"));
    Ok(())
}

#[test]
fn newly_added_home_targets_keep_expected_ages() -> Result<(), &'static str> {
    for (id, hours) in [("xcode-device-support", 72), ("chrome-on-device-model", 24)] {
        let target = catalog()
            .iter()
            .find(|target| target.id == id)
            .ok_or("target missing")?;
        assert_eq!(target.minimum_age.as_secs(), hours * 3_600, "{id}");
    }
    Ok(())
}

mod auto_policy {
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use cleaner_domain::{
        AutoCleanSettings, AutoRamDecision, AutoRamSettings, AutoRamState, GIB, MemorySnapshot,
        ProcessSnapshot,
    };

    use crate::{app_bundle, is_never_quit, plan_ram_relief, validate_auto_settings};

    fn at(seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_800_000_000 + seconds, 0)
            .single()
            .unwrap_or_default()
    }

    fn memory(available_mib: u64) -> MemorySnapshot {
        MemorySnapshot {
            total_bytes: 19 * GIB,
            free_bytes: 0,
            available_bytes: available_mib * 1_048_576,
            used_bytes: 0,
            swap_used_bytes: 0,
            reclaimable_bytes: 0,
            compressed_bytes: 0,
            file_backed_bytes: 0,
            pressure: cleaner_domain::MemoryPressureLevel::Critical,
        }
    }

    fn process(pid: u32, exe: &str, mib: u64, protected: bool) -> ProcessSnapshot {
        ProcessSnapshot {
            pid,
            start_time_unix: 1,
            name: exe.rsplit('/').next().unwrap_or_default().to_owned(),
            executable: Some(exe.to_owned()),
            memory_bytes: mib * 1_048_576,
            cpu_percent: 0.0,
            protected,
            protection_reason: None,
        }
    }

    fn settings(apps: &[&str]) -> AutoRamSettings {
        AutoRamSettings {
            enabled: true,
            min_available_bytes: 2 * GIB,
            sustained_secs: 120,
            cooldown_secs: 180,
            quit_apps: apps.iter().map(|app| (*app).to_owned()).collect(),
        }
    }

    const CHROME: &str = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
    const CHROME_HELPER: &str = "/Applications/Google Chrome.app/Contents/Frameworks/Google Chrome Framework.framework/Helpers/Google Chrome Helper.app/Contents/MacOS/Google Chrome Helper";
    const TEAMS: &str = "/Applications/Microsoft Teams.app/Contents/MacOS/MSTeams";

    #[test]
    fn app_bundles_come_only_from_applications_folders() {
        assert_eq!(
            app_bundle(CHROME),
            Some((
                "Google Chrome".to_owned(),
                "/Applications/Google Chrome.app".to_owned()
            ))
        );
        // A helper nested inside the bundle still belongs to the outer app.
        assert_eq!(
            app_bundle(CHROME_HELPER).map(|(name, _)| name),
            Some("Google Chrome".to_owned())
        );
        assert!(app_bundle("/Users/me/Applications/Foo.app/Contents/MacOS/Foo").is_some());
        assert!(app_bundle("/System/Applications/Safari.app/Contents/MacOS/Safari").is_none());
        assert!(app_bundle("/Users/me/Downloads/Evil.app/Contents/MacOS/Evil").is_none());
        assert!(app_bundle("/usr/bin/vm_stat").is_none());
    }

    #[test]
    fn protected_and_developer_apps_can_never_be_listed() {
        for name in [
            "Finder",
            "Docker Desktop",
            "Xcode",
            "Claude",
            "Mac Cleaner",
            "Terminal",
        ] {
            assert!(is_never_quit(name), "{name}");
        }
        assert!(!is_never_quit("Google Chrome"));
        assert!(!is_never_quit("Barcode Scanner"));
        assert!(!is_never_quit("Autumn Leaves"));
        assert!(!is_never_quit("Spider Solitaire"));
        assert!(!is_never_quit("Barcode Scanner"));
        let mut bad = AutoCleanSettings::default();
        bad.ram.quit_apps = vec!["Docker Desktop".to_owned()];
        assert!(validate_auto_settings(bad).is_err());
        let mut slash = AutoCleanSettings::default();
        slash.ram.quit_apps = vec!["../Evil".to_owned()];
        assert!(validate_auto_settings(slash).is_err());
    }

    #[test]
    fn settings_are_clamped_deduped_and_checked_against_the_catalog()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut input = AutoCleanSettings::default();
        input.ram.min_available_bytes = 1;
        input.ram.sustained_secs = 1;
        input.ram.cooldown_secs = 999_999;
        input.ram.quit_apps = vec!["Slack".to_owned(), " slack ".to_owned(), String::new()];
        input.storage.interval_hours = 0;
        input.storage.min_idle_hours = 1;
        input.storage.target_ids = vec!["npm-npx-cache".to_owned(), "npm-npx-cache".to_owned()];
        let clean = validate_auto_settings(input)?;
        assert_eq!(clean.ram.min_available_bytes, GIB / 2);
        assert_eq!(clean.ram.sustained_secs, 30);
        assert_eq!(clean.ram.cooldown_secs, 3_600);
        assert_eq!(clean.ram.quit_apps, ["Slack"]);
        assert_eq!(clean.storage.interval_hours, 1);
        assert_eq!(clean.storage.min_idle_hours, 24);
        assert_eq!(clean.storage.target_ids, ["npm-npx-cache"]);

        let mut unknown = AutoCleanSettings::default();
        unknown.storage.target_ids = vec!["home-directory".to_owned()];
        assert!(validate_auto_settings(unknown).is_err());
        Ok(())
    }

    #[test]
    fn nothing_happens_while_disabled_or_healthy() {
        let procs = [process(1, CHROME, 900, false)];
        let mut state = AutoRamState::default();
        let mut off = settings(&["Google Chrome"]);
        off.enabled = false;
        assert_eq!(
            plan_ram_relief(&off, &memory(512), &procs, &mut state, at(0)),
            AutoRamDecision::Disabled
        );
        assert_eq!(
            plan_ram_relief(
                &settings(&["Google Chrome"]),
                &memory(6144),
                &procs,
                &mut state,
                at(0)
            ),
            AutoRamDecision::Healthy
        );
        assert!(state.low_since.is_none());
    }

    #[test]
    fn memory_must_stay_low_before_the_biggest_listed_app_is_chosen() {
        let procs = [
            process(10, CHROME, 700, false),
            process(11, CHROME_HELPER, 900, false),
            process(20, TEAMS, 800, false),
            process(
                30,
                "/Applications/Unlisted.app/Contents/MacOS/Unlisted",
                5_000,
                false,
            ),
        ];
        let config = settings(&["Google Chrome", "Microsoft Teams"]);
        let mut state = AutoRamState::default();

        // First low reading only starts the clock.
        assert_eq!(
            plan_ram_relief(&config, &memory(1024), &procs, &mut state, at(0)),
            AutoRamDecision::Waiting {
                secs_remaining: 120
            }
        );
        assert_eq!(
            plan_ram_relief(&config, &memory(1024), &procs, &mut state, at(60)),
            AutoRamDecision::Waiting { secs_remaining: 60 }
        );
        // A recovery resets the clock.
        assert_eq!(
            plan_ram_relief(&config, &memory(4096), &procs, &mut state, at(70)),
            AutoRamDecision::Healthy
        );
        assert_eq!(
            plan_ram_relief(&config, &memory(1024), &procs, &mut state, at(80)),
            AutoRamDecision::Waiting {
                secs_remaining: 120
            }
        );

        let decision = plan_ram_relief(&config, &memory(1024), &procs, &mut state, at(200));
        // Chrome's helper memory is added to the app, so it beats Teams; the
        // larger unlisted app is never considered.
        let AutoRamDecision::Quit { target } = decision else {
            panic!("expected a quit, got {decision:?}");
        };
        assert_eq!(target.app_name, "Google Chrome");
        assert_eq!(target.app_path, "/Applications/Google Chrome.app");
        assert_eq!(target.bytes, 1_600 * 1_048_576);
        assert_eq!(target.pids, [10, 11]);
    }

    #[test]
    fn cooldown_follows_a_recorded_quit_and_refusing_apps_are_not_retried_forever() {
        let procs = [
            process(10, CHROME, 900, false),
            process(20, TEAMS, 300, false),
        ];
        let config = settings(&["Google Chrome", "Microsoft Teams"]);
        let mut state = AutoRamState {
            low_since: Some(at(0)),
            last_quit_at: Some(at(100)),
            recent_quits: vec![("Google Chrome".to_owned(), at(100))],
        };
        assert_eq!(
            plan_ram_relief(&config, &memory(1024), &procs, &mut state, at(200)),
            AutoRamDecision::Cooldown { secs_remaining: 80 }
        );

        // After the cooldown Chrome is tried a second time…
        let AutoRamDecision::Quit { target } =
            plan_ram_relief(&config, &memory(1024), &procs, &mut state, at(400))
        else {
            panic!("expected a second attempt");
        };
        assert_eq!(target.app_name, "Google Chrome");

        // …but after two requests it is left alone and the next app is used.
        state
            .recent_quits
            .push(("Google Chrome".to_owned(), at(390)));
        state.last_quit_at = Some(at(390));
        let AutoRamDecision::Quit { target } =
            plan_ram_relief(&config, &memory(1024), &procs, &mut state, at(700))
        else {
            panic!("expected Teams");
        };
        assert_eq!(target.app_name, "Microsoft Teams");

        // Old requests expire, so Chrome becomes eligible again.
        let later = at(700) + Duration::minutes(31);
        state.low_since = Some(at(0));
        let AutoRamDecision::Quit { target } =
            plan_ram_relief(&config, &memory(1024), &procs, &mut state, later)
        else {
            panic!("expected Chrome again");
        };
        assert_eq!(target.app_name, "Google Chrome");
    }

    #[test]
    fn protected_helpers_and_empty_lists_never_produce_a_quit() {
        let mut state = AutoRamState {
            low_since: Some(at(0)),
            ..AutoRamState::default()
        };
        let none_listed = settings(&[]);
        assert!(matches!(
            plan_ram_relief(
                &none_listed,
                &memory(1024),
                &[process(1, CHROME, 9, false)],
                &mut state,
                at(500)
            ),
            AutoRamDecision::NoTargets { .. }
        ));

        let config = settings(&["Google Chrome"]);
        // One protected process inside the app protects the whole app.
        let guarded = [
            process(1, CHROME, 900, false),
            process(2, CHROME_HELPER, 10, true),
        ];
        assert!(matches!(
            plan_ram_relief(&config, &memory(1024), &guarded, &mut state, at(500)),
            AutoRamDecision::NoTargets { .. }
        ));
        // Not running at all.
        assert!(matches!(
            plan_ram_relief(&config, &memory(1024), &[], &mut state, at(500)),
            AutoRamDecision::NoTargets { .. }
        ));
    }
}
