---
domain_id: dom-ai-assistant
slug: ai-assistant
status: retired
repo_ids: [mac-cleaner]
related_domain_slugs: [memory-optimization]
created_at: 2026-04-29
updated_at: 2026-11-16
---

# ai-assistant

<!-- updated: 2026-11-16 — retired during the Swift/Rust rewrite. -->

Historical terminal assistant for RAM/process advice. Its Python/OpenAI implementation and launcher were removed; no model provider or AI execution path ships in the current product.

## Current contract

- Do not reintroduce an AI provider, background decision loop, or model-triggered cleanup without a new stream and explicit safety review.
- The Rust policy engine and native user confirmation remain authoritative.
- Historical source remains available through Git history only.

