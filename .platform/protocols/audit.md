# Protocol: Stream audit / analysis

> When the user asks to audit, analyze or check the state of a stream or its implementation.

Whenever the user asks to audit, analyze, or check the state of a stream or its implementation (e.g. "audit the stream", "analyze what we've done on X", "check the state of \<feature\>", or references a `work/*.md` file explicitly), you MUST:

1. **Read `.platform/workflow.md` first** — before touching any agent results or synthesizing anything.
2. **Follow the Stream / Feature Analysis Protocol exactly** as written there — no shortcuts, no partial synthesis before all agents finish, no skipping the standardized template or the stream-file anchor step.

This applies even when resuming after context compaction. A compressed context summary is NOT a substitute for reading the protocol spec.
