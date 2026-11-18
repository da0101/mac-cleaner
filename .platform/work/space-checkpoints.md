---
stream_id: stream-space-checkpoints
slug: space-checkpoints
type: feature
status: in-progress
agent_owner: claude-code
domain_slugs: [storage-forensics]
repo_ids: [mac-cleaner]
base_branch: develop
git_branch: develop
created_at: 2026-11-18
updated_at: 2026-11-18
closure_approved: false
---

# space-checkpoints

## Scope
- Overview box "Where did my space go": free-space change since the last checkpoint, itemised by tracked location, plus an honest unexplained remainder.
- A checkpoint is the existing storage baseline; one is set automatically after every cleanup and on demand.
- Memory swap is measured as a read-only tracked item (it sits on a separate volume the folder scan cannot see).
- Menu bar panel: compact vitals, space-since-checkpoint line, Auto-Clean switches, busiest-app icons, one-tap shortcuts.
- System temp folder and ~/.claude are tracked read-only so growth there is itemised.
- Out of scope: deleting anything, Time Machine snapshot sizes, deep "what changed" folder scan (possible follow-up).

## Done criteria
- [x] Swap appears as a tracked, protected storage item with deltas against a checkpoint.
- [x] Overview shows the box with free-space delta, top movers, unexplained remainder, checkpoint buttons.
- [x] Cleanup finishing sets a new checkpoint automatically.
- [x] `cargo fmt`, `clippy -D warnings`, `cargo test --workspace` pass.
- [ ] Owner confirms the box answers "where did the space go" on real use.
- [ ] `.platform/memory/log.md` appended
- [ ] `decisions.md` updated if any architectural choices were made

## Key decisions
2026-11-18 — Reuse the existing snapshot baseline as the checkpoint instead of a new store — comparison and accounting gap were already computed and persisted.
2026-11-18 — Swap is a Protected hotspot with a fixed id — it is reported, never offered for cleanup.
