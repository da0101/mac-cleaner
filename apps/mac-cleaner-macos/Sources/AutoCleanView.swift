import AppKit
import SwiftUI

/// Cleaning that runs on its own, inside limits the owner chooses here.
/// Rust makes every decision; this screen only edits the opt-in lists.
struct AutoCleanView: View {
    @EnvironmentObject private var model: AppModel
    @State private var runningApps: [RunningApp] = []

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                header
                statusBanner
                memoryCard
                storageCard
                activityCard
            }
            .padding(.horizontal, 26)
            .padding(.top, 22)
            .padding(.bottom, 28)
        }
        .navigationTitle("Auto-Clean")
        .task {
            refreshRunningApps()
            await model.loadAutoClean()
        }
    }

    // MARK: Header

    private var header: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("AUTO-CLEAN")
                .font(.system(size: 10, weight: .bold, design: .rounded))
                .tracking(1.8)
                .foregroundStyle(CleanerTheme.mint)
            Text("Keep the Mac clean on its own")
                .font(.system(size: 29, weight: .bold, design: .rounded))
                .tracking(-0.6)
            Text("Everything here is off until you turn it on, and it only touches what you pick. Developer tools, Docker, and system apps can never be quit.")
                .foregroundStyle(.secondary)
        }
    }

    private var statusBanner: some View {
        let active = model.autoSettings.ram.enabled || model.autoSettings.storage.enabled
        return HStack(spacing: 12) {
            Image(systemName: active ? "bolt.circle.fill" : "bolt.slash.circle")
                .font(.system(size: 24))
                .foregroundStyle(active ? CleanerTheme.mint : .secondary)
            VStack(alignment: .leading, spacing: 2) {
                Text(active ? "Auto-clean is running" : "Auto-clean is off")
                    .font(.headline)
                Text(model.autoStatus)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
        }
        .padding(14)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 17, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 17, style: .continuous)
                .stroke((active ? CleanerTheme.mint : Color.secondary).opacity(0.18), lineWidth: 1)
        }
    }

    // MARK: Memory

    private var memoryCard: some View {
        card(tint: CleanerTheme.coral) {
            VStack(alignment: .leading, spacing: 14) {
                Toggle(isOn: binding(\.ram.enabled)) {
                    VStack(alignment: .leading, spacing: 3) {
                        Text("Free memory automatically").font(.headline)
                        Text("When available memory stays low, Mac Cleaner asks the apps you list below to quit, biggest first. It is a normal quit, like ⌘Q, so unsaved documents still ask to save.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                .toggleStyle(.switch)

                HStack(spacing: 18) {
                    Picker("When available memory is below", selection: binding(\.ram.minAvailableBytes)) {
                        ForEach([1, 2, 3, 4], id: \.self) { gb in
                            Text("\(gb) GB").tag(UInt64(gb) * 1_073_741_824)
                        }
                    }
                    Picker("and stays low for", selection: binding(\.ram.sustainedSecs)) {
                        Text("1 min").tag(UInt64(60))
                        Text("2 min").tag(UInt64(120))
                        Text("5 min").tag(UInt64(300))
                    }
                }
                .disabled(!model.autoSettings.ram.enabled)

                Divider()
                VStack(alignment: .leading, spacing: 4) {
                    Text("Apps Mac Cleaner may quit")
                        .font(.subheadline.weight(.semibold))
                    Text("Only apps you tick are ever asked to quit. Each app is tried at most twice in 30 minutes, then left alone if it refuses.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                if appRows.isEmpty {
                    Text("No other apps are running right now.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                ForEach(appRows) { app in
                    Toggle(isOn: appBinding(app.name)) {
                        HStack(spacing: 9) {
                            if let icon = app.icon {
                                Image(nsImage: icon).resizable().frame(width: 20, height: 20)
                            }
                            Text(app.name)
                            if !app.running {
                                Text("not running").font(.caption2).foregroundStyle(.tertiary)
                            }
                        }
                    }
                }
                HStack {
                    Spacer()
                    Button("Refresh running apps", systemImage: "arrow.clockwise") { refreshRunningApps() }
                        .buttonStyle(.borderless)
                        .font(.caption)
                }
            }
        }
    }

    // MARK: Storage

    private var storageCard: some View {
        card(tint: CleanerTheme.azure) {
            VStack(alignment: .leading, spacing: 14) {
                Toggle(isOn: binding(\.storage.enabled)) {
                    VStack(alignment: .leading, spacing: 3) {
                        Text("Clean storage on a schedule").font(.headline)
                        Text("Removes the categories you tick below, permanently, through the same safe steps as manual cleanup: exact plan, re-check, receipt. Anything changed in the last 24 hours is never removed.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                .toggleStyle(.switch)

                HStack(spacing: 18) {
                    Picker("Run every", selection: binding(\.storage.intervalHours)) {
                        ForEach([1, 3, 6, 12, 24], id: \.self) { hours in
                            Text(hours == 1 ? "hour" : "\(hours) hours").tag(UInt64(hours))
                        }
                    }
                    Text("Also hourly while free space is under 40 GB.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .disabled(!model.autoSettings.storage.enabled)

                Divider()
                HStack {
                    Text("Categories to clean")
                        .font(.subheadline.weight(.semibold))
                    Spacer()
                    Button("Choose recommended caches") {
                        change { $0.storage.targetIds = Self.recommended }
                    }
                    .buttonStyle(.borderless)
                    .font(.caption)
                    Button("Clear") { change { $0.storage.targetIds = [] } }
                        .buttonStyle(.borderless)
                        .font(.caption)
                }
                ForEach(Self.categories) { category in
                    Toggle(isOn: categoryBinding(category.id)) {
                        VStack(alignment: .leading, spacing: 1) {
                            Text(category.title)
                            Text(category.detail).font(.caption).foregroundStyle(.secondary)
                        }
                    }
                }
                if model.autoSettings.storage.enabled && model.autoSettings.storage.targetIds.isEmpty {
                    Label("Pick at least one category, or nothing will be cleaned.", systemImage: "exclamationmark.triangle")
                        .font(.caption)
                        .foregroundStyle(CleanerTheme.amber)
                }
            }
        }
    }

    // MARK: Activity

    private var activityCard: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Recent auto-clean activity").font(.headline)
            if model.autoLog.isEmpty {
                Text("Nothing has run yet. Every automatic quit and cleanup is recorded here.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            ForEach(model.autoLog) { entry in
                HStack(alignment: .top, spacing: 10) {
                    Image(systemName: symbol(for: entry.kind))
                        .foregroundStyle(tint(for: entry.kind))
                        .frame(width: 20)
                    VStack(alignment: .leading, spacing: 1) {
                        Text(entry.summary)
                        Text(Self.time(entry.at)).font(.caption2).foregroundStyle(.secondary)
                    }
                    Spacer()
                    if entry.bytes > 0 {
                        Text(Formatting.bytes(entry.bytes)).font(.callout.monospacedDigit())
                    }
                }
                .padding(.vertical, 4)
            }
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 17, style: .continuous))
    }

    // MARK: Helpers

    private func card<Content: View>(tint: Color, @ViewBuilder content: () -> Content) -> some View {
        content()
            .padding(16)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 17, style: .continuous))
            .overlay { RoundedRectangle(cornerRadius: 17, style: .continuous).stroke(tint.opacity(0.16), lineWidth: 1) }
    }

    private func change(_ mutate: (inout AutoCleanSettings) -> Void) {
        var next = model.autoSettings
        mutate(&next)
        Task { await model.saveAutoSettings(next) }
    }

    private func binding<Value>(_ keyPath: WritableKeyPath<AutoCleanSettings, Value>) -> Binding<Value> {
        Binding(
            get: { model.autoSettings[keyPath: keyPath] },
            set: { value in change { $0[keyPath: keyPath] = value } }
        )
    }

    private func appBinding(_ name: String) -> Binding<Bool> {
        Binding(
            get: { model.autoSettings.ram.quitApps.contains { $0.caseInsensitiveCompare(name) == .orderedSame } },
            set: { on in
                change { settings in
                    settings.ram.quitApps.removeAll { $0.caseInsensitiveCompare(name) == .orderedSame }
                    if on { settings.ram.quitApps.append(name) }
                }
            }
        )
    }

    private func categoryBinding(_ id: String) -> Binding<Bool> {
        Binding(
            get: { model.autoSettings.storage.targetIds.contains(id) },
            set: { on in
                change { settings in
                    settings.storage.targetIds.removeAll { $0 == id }
                    if on { settings.storage.targetIds.append(id) }
                }
            }
        )
    }

    private var appRows: [RunningApp] {
        // Filtered here, not when apps are collected, so it follows the protected list once it loads.
        var rows = runningApps.filter { !isProtected($0.name) }
        for name in model.autoSettings.ram.quitApps
        where !rows.contains(where: { $0.name.caseInsensitiveCompare(name) == .orderedSame }) {
            rows.append(RunningApp(name: name, icon: nil, running: false))
        }
        return rows.sorted { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }
    }

    /// Regular, user-visible apps from an Applications folder. Rust still
    /// refuses protected apps (Docker, editors, Finder…) when settings are saved.
    private func refreshRunningApps() {
        let allowed = ["/Applications/", NSHomeDirectory() + "/Applications/"]
        runningApps = NSWorkspace.shared.runningApplications.compactMap { app in
            guard app.activationPolicy == .regular,
                  let path = app.bundleURL?.path,
                  allowed.contains(where: { path.hasPrefix($0) }),
                  app.bundleIdentifier != Bundle.main.bundleIdentifier,
                  let name = app.localizedName else { return nil }
            return RunningApp(name: name, icon: app.icon, running: true)
        }
    }

    /// Protected apps (editors, Docker, system apps) are never offered.
    private func isProtected(_ name: String) -> Bool {
        let lower = name.lowercased()
        return model.autoProtectedApps.contains { lower == $0 || ($0.count > 4 && lower.contains($0)) }
    }

    private func symbol(for kind: String) -> String {
        switch kind {
        case "ram_quit": "memorychip"
        case "storage_clean": "trash"
        default: "exclamationmark.triangle"
        }
    }

    private func tint(for kind: String) -> Color {
        switch kind {
        case "ram_quit": CleanerTheme.coral
        case "storage_clean": CleanerTheme.mint
        default: CleanerTheme.amber
        }
    }

    private static func time(_ iso: String) -> String {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        let date = formatter.date(from: iso) ?? ISO8601DateFormatter().date(from: iso)
        return date.map { $0.formatted(date: .abbreviated, time: .shortened) } ?? iso
    }

    private static let recommended = [
        "npm-npx-cache", "npm-content-cache", "yarn-berry-cache", "puppeteer-browser-cache",
        "project-yarn-offline-cache", "project-ios-build", "project-xcode-derived-data", "xcode-derived-data",
        "vscode-updater-cache", "postman-updater-cache", "homebrew-download-cache", "pip-download-cache",
        "playwright-browser-cache", "cypress-binary-cache", "bun-package-cache", "maestro-test-output",
        "xctest-clone-devices",
    ]

    private static let categories: [AutoCategory] = [
        AutoCategory(id: "npm-npx-cache", title: "npx tool cache", detail: "Downloaded again the next time npx runs a tool."),
        AutoCategory(id: "npm-content-cache", title: "npm package cache", detail: "Re-downloaded by npm when needed."),
        AutoCategory(id: "yarn-berry-cache", title: "Yarn shared cache", detail: "Re-downloaded by the next yarn install."),
        AutoCategory(id: "project-yarn-offline-cache", title: "Yarn caches inside projects", detail: "Only when Git ignores them. The next yarn install downloads packages again."),
        AutoCategory(id: "puppeteer-browser-cache", title: "Puppeteer browsers", detail: "Downloaded again by the next automated test run."),
        AutoCategory(id: "project-ios-build", title: "iOS build folders", detail: "Only when Git ignores them and they are 3+ days idle. The next build recreates them."),
        AutoCategory(id: "project-xcode-derived-data", title: "Xcode build output inside projects", detail: "Only when Git ignores it and it is 3+ days idle."),
        AutoCategory(id: "xcode-derived-data", title: "Xcode DerivedData", detail: "Projects untouched for 14+ days. Xcode rebuilds and re-indexes."),
        AutoCategory(id: "xcode-device-support", title: "Xcode device symbols", detail: "Copied from the iPhone again next time it is connected (a few minutes)."),
        AutoCategory(id: "chrome-on-device-model", title: "Chrome on-device model", detail: "Chrome downloads it again when a feature needs it."),
        AutoCategory(id: "vscode-updater-cache", title: "VS Code update leftovers", detail: "Old downloaded update packages. 3+ days idle."),
        AutoCategory(id: "postman-updater-cache", title: "Postman update leftovers", detail: "Old downloaded update packages. 3+ days idle."),
        AutoCategory(id: "chrome-browser-cache", title: "Chrome browser cache", detail: "Only when Chrome has not run for a day. Pages and images load again."),
        AutoCategory(id: "homebrew-download-cache", title: "Homebrew downloads", detail: "Old bottles and sources. 3+ days idle."),
        AutoCategory(id: "pip-download-cache", title: "pip download cache", detail: "Python packages download again when needed."),
        AutoCategory(id: "playwright-browser-cache", title: "Playwright browsers", detail: "Downloaded again by the next Playwright test run."),
        AutoCategory(id: "cypress-binary-cache", title: "Cypress binaries", detail: "Downloaded again by the next Cypress run."),
        AutoCategory(id: "bun-package-cache", title: "Bun package cache", detail: "Re-downloaded by the next bun install."),
        AutoCategory(id: "huggingface-model-cache", title: "Hugging Face models", detail: "Models download again, which can be large. 7+ days idle."),
        AutoCategory(id: "maestro-test-output", title: "Maestro test output", detail: "Old screenshots, recordings and logs. 3+ days idle."),
        AutoCategory(id: "xctest-clone-devices", title: "Xcode test simulator clones", detail: "Created by parallel UI tests. 3+ days idle."),
        AutoCategory(id: "cocoapods-download-cache", title: "CocoaPods download cache", detail: "Pods download again on the next install."),
        AutoCategory(id: "project-flutter-build", title: "Flutter build output", detail: "The next Flutter build recreates it, which can take minutes."),
        AutoCategory(id: "project-rust-target", title: "Rust target folders", detail: "The next cargo build recreates them, which can take many minutes."),
        AutoCategory(id: "project-node-modules", title: "node_modules folders", detail: "You must run install again before using the project. Use with care."),
    ]
}

private struct RunningApp: Identifiable {
    let name: String
    let icon: NSImage?
    let running: Bool
    var id: String { name.lowercased() }
}

private struct AutoCategory: Identifiable {
    let id: String
    let title: String
    let detail: String
}
