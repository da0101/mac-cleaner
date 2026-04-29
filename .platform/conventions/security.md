# Security Conventions — mac-cleaner

- Never commit `.env`, tokens, cleanup history, virtualenvs, caches, or local AI workspace files.
- Keep dashboard bound to `127.0.0.1`.
- Remove permissive CORS unless a specific local client needs it.
- Destructive actions require explicit user intent.
- System Data parent paths are report-only.
- Docker prune, Trash emptying, logs, archives, and package stores are review-required.

