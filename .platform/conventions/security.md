# Security Conventions — mac-cleaner

- Never commit `.env`, tokens, cleanup history, SQLite runtime state, caches, or local AI workspace files.
- Open no HTTP listener or web view; Swift resolves the engine only from the signed application bundle.
- Swift may invoke fixed engine commands with opaque IDs and bounded options, never arbitrary filesystem paths or shell fragments.
- Destructive actions require a fresh Rust-owned plan, native user confirmation, approval, identity revalidation, and a durable receipt.
- System Data parent paths and storage-forensics findings are report-only.
- Repositories, credentials, databases, SDKs, simulator runtimes, package stores, user content, and unknown paths are protected.
- Tests that mutate files operate only in temporary fake homes; live QA is read-only unless the owner confirms an exact plan.

