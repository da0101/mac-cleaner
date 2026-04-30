---
domain_id: dom-storage-cleanup
slug: storage-cleanup
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [memory-optimization, local-dashboard]
created_at: 2026-04-29
updated_at: 2026-04-29
---

# storage-cleanup

Finds known cleanable garbage and reports mixed System Data contributors.

## Key files

- `scanner.py` — shared target catalog, safety tiers, allocated disk usage, cleanup helpers.
- `cleaner.py` — terminal UI.
- `server.py` — dashboard/API integration.

## Locked contracts

- Known safe/review targets may be cleanable.
- System Data parent paths are report-only.
- Sizes use allocated blocks so sparse VM/container files are not overcounted.
- Never default-delete user documents, source repos, credentials, app support databases, or unknown data.
- Dart/Flutter's global Pub package store (`~/.pub-cache`) is report-only/protected; use Pub/Flutter commands for maintenance instead of deleting it from mac-cleaner.
