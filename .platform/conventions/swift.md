# Swift/macOS Conventions — mac-cleaner

- Use SwiftUI/AppKit and Apple system frameworks only; do not add a web view, cross-platform UI toolkit, or third-party UI dependency.
- `AppModel` owns observable UI state; `EngineClient` is the sole Rust boundary and resolves its helper through `Bundle`.
- Model protocol payloads with typed `Decodable` structures; ignore additive backend fields but fail on missing required fields.
- Destructive UI starts with a Rust-prepared plan and uses a native alert before an execute command.
- Prefer native `NavigationSplitView`, `Table`, `List`, `Form`, materials, SF Symbols, keyboard shortcuts, and accessibility labels.
- Maintain macOS 13 compatibility until the deployment target is intentionally raised.
