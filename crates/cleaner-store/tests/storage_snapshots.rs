use chrono::{Duration, Utc};
use cleaner_domain::{STORAGE_CATALOG_VERSION, StorageAnalysis, StoragePressure};
use cleaner_store::Store;
use uuid::Uuid;

fn snapshot(index: i64) -> StorageAnalysis {
    StorageAnalysis {
        snapshot_id: Uuid::new_v4(),
        catalog_version: STORAGE_CATALOG_VERSION,
        captured_at: Utc::now() + Duration::seconds(index),
        total_bytes: 1_000,
        available_bytes: 500,
        pressure: StoragePressure::Healthy,
        reviewable_bytes: 0,
        hotspots: Vec::new(),
        warnings: Vec::new(),
        comparison: None,
    }
}

#[test]
fn snapshots_round_trip_and_latest_is_ordered_by_capture_time()
-> Result<(), Box<dyn std::error::Error>> {
    let store = Store::open_in_memory()?;
    let first = snapshot(1);
    let second = snapshot(2);
    store.save_storage_snapshot(&first)?;
    store.save_storage_snapshot(&second)?;

    assert_eq!(store.storage_snapshot(first.snapshot_id)?, Some(first));
    assert_eq!(store.latest_storage_snapshot()?, Some(second));
    Ok(())
}

#[test]
fn retention_is_bounded_and_preserves_the_selected_baseline()
-> Result<(), Box<dyn std::error::Error>> {
    let store = Store::open_in_memory()?;
    let baseline = snapshot(0);
    store.save_storage_snapshot(&baseline)?;
    store.set_storage_baseline(baseline.snapshot_id)?;
    for index in 1..=35 {
        store.save_storage_snapshot(&snapshot(index))?;
    }

    assert_eq!(store.storage_snapshot_count()?, 30);
    assert_eq!(store.storage_baseline()?, Some(baseline));
    Ok(())
}

#[test]
fn unknown_snapshot_cannot_become_a_baseline() -> Result<(), Box<dyn std::error::Error>> {
    let store = Store::open_in_memory()?;
    assert!(store.set_storage_baseline(Uuid::new_v4()).is_err());
    Ok(())
}
