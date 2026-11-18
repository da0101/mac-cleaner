import AppKit
import SwiftUI

/// Quick-access panel: live vitals, where space went, the Auto-Clean switches,
/// the busiest apps as icons, and one-tap shortcuts. Everything here is a glance
/// or a single action; the full screens live in the main window.
struct MenuBarView: View {
    @Environment(\.openWindow) private var openWindow
    @EnvironmentObject private var model: AppModel

    var body: some View {
        VStack(spacing: 12) {
            header

            if let snapshot = model.snapshot {
                vitals(snapshot)
                spaceLine(snapshot)
            } else {
                HStack(spacing: 10) {
                    ProgressView().controlSize(.small)
                    Text("Reading system status…").foregroundStyle(.secondary)
                    Spacer()
                }
                .padding(12)
                .background(.quaternary.opacity(0.55), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            }

            autoCleanCard
            appStrip
            shortcuts

            Divider()
            footer
        }
        .padding(14)
        .frame(width: 350)
        .background(Color(nsColor: .windowBackgroundColor))
        .task { model.startMonitoring() }
        .task {
            // Sampling costs about a second of CPU, so it only runs while this panel is open.
            while !Task.isCancelled {
                await model.refreshEnergy()
                try? await Task.sleep(for: .seconds(6))
            }
        }
    }

    // MARK: Header

    private var header: some View {
        HStack(spacing: 9) {
            CleanerBrandMark()
                .frame(width: 30, height: 30)
                .accessibilityHidden(true)
            Text("Mac Cleaner").font(.headline)
            Spacer()
            if model.isBusy {
                ProgressView().controlSize(.small).accessibilityLabel("Mac Cleaner is working")
            } else {
                Label(statusText, systemImage: "circle.fill")
                    .labelStyle(StatusDotStyle(tint: statusColor))
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }

    // MARK: Vitals

    private func vitals(_ snapshot: SystemSnapshot) -> some View {
        HStack(spacing: 8) {
            chip(
                symbol: "internaldrive",
                value: Formatting.compactBytes(snapshot.disk.availableBytes),
                label: "free",
                tint: storageTint(snapshot.disk.availableBytes, total: snapshot.disk.totalBytes)
            )
            chip(
                symbol: "memorychip",
                value: Formatting.compactBytes(snapshot.memory.availableBytes),
                label: "RAM free",
                tint: memoryTint(snapshot.memory.pressure)
            )
            chip(
                symbol: "arrow.left.arrow.right",
                value: Formatting.compactBytes(snapshot.memory.swapUsedBytes),
                label: "swap",
                tint: snapshot.memory.swapUsedBytes > 8 * 1_073_741_824 ? .orange : CleanerTheme.violet
            )
            chip(
                symbol: "cpu",
                value: Formatting.cpuPercent(snapshot.cpu.usagePercent),
                label: "CPU",
                tint: cpuTint(snapshot.cpu.pressure)
            )
        }
    }

    private func chip(symbol: String, value: String, label: String, tint: Color) -> some View {
        VStack(spacing: 2) {
            Image(systemName: symbol).font(.system(size: 11, weight: .semibold)).foregroundStyle(tint)
            Text(value)
                .font(.system(size: 14, weight: .bold, design: .rounded))
                .monospacedDigit()
                .lineLimit(1)
                .minimumScaleFactor(0.7)
            Text(label).font(.system(size: 9)).foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 8)
        .background(Color(nsColor: .controlBackgroundColor).opacity(0.96), in: RoundedRectangle(cornerRadius: 11, style: .continuous))
        .overlay { RoundedRectangle(cornerRadius: 11, style: .continuous).stroke(tint.opacity(0.22), lineWidth: 1) }
    }

    /// Same math as the Overview box: live free space against the checkpoint's free space.
    private func spaceLine(_ snapshot: SystemSnapshot) -> some View {
        let change = spaceChange(snapshot)
        return Button {
            openMainWindow(select: .overview)
        } label: {
            HStack(spacing: 8) {
                Image(systemName: "flag.fill").foregroundStyle(CleanerTheme.amber).font(.caption)
                if let change {
                    Text(Formatting.signedBytes(change))
                        .font(.callout.weight(.bold).monospacedDigit())
                        .foregroundStyle(change < 0 ? CleanerTheme.coral : CleanerTheme.mint)
                    Text("since checkpoint").font(.caption).foregroundStyle(.secondary).lineLimit(1)
                } else {
                    Text("No checkpoint yet").font(.caption).foregroundStyle(.secondary)
                }
                Spacer(minLength: 4)
                Text("Details").font(.caption2).foregroundStyle(.tertiary)
                Image(systemName: "chevron.right").font(.system(size: 9)).foregroundStyle(.tertiary)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 9)
            .background(Color(nsColor: .controlBackgroundColor).opacity(0.96), in: RoundedRectangle(cornerRadius: 11, style: .continuous))
        }
        .buttonStyle(.plain)
        .help("Open the Overview to see exactly where your space went")
    }

    private func spaceChange(_ snapshot: SystemSnapshot) -> Int64? {
        guard let storage = model.storage else { return nil }
        let live = Int64(clamping: snapshot.disk.availableBytes)
        if model.justCheckpointed {
            return live &- Int64(clamping: storage.availableBytes)
        }
        guard let comparison = storage.comparison, comparison.kind == "baseline" else { return nil }
        return live &- (Int64(clamping: storage.availableBytes) &+ comparison.volumeUsedDeltaBytes)
    }

    // MARK: Auto-Clean

    private var autoCleanCard: some View {
        VStack(spacing: 6) {
            HStack(spacing: 6) {
                Image(systemName: "bolt.circle.fill").foregroundStyle(CleanerTheme.mint)
                Text("Auto-Clean").font(.caption.weight(.semibold))
                Spacer()
                Button("Settings") { openMainWindow(select: .autoClean) }
                    .buttonStyle(.plain)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            switchRow(
                symbol: "memorychip",
                title: "Free memory when low",
                note: ramNote,
                isOn: autoBinding(\.ram.enabled)
            )
            switchRow(
                symbol: "trash",
                title: "Clean storage on a schedule",
                note: storageNote,
                isOn: autoBinding(\.storage.enabled)
            )
        }
        .controlSize(.small)
        .padding(11)
        .background(Color(nsColor: .controlBackgroundColor).opacity(0.96), in: RoundedRectangle(cornerRadius: 11, style: .continuous))
    }

    /// One aligned row: icon and title on the left, the switch on the right, a plain-language note below.
    private func switchRow(symbol: String, title: String, note: String, isOn: Binding<Bool>) -> some View {
        HStack(alignment: .center, spacing: 8) {
            Image(systemName: symbol)
                .font(.system(size: 12))
                .foregroundStyle(.secondary)
                .frame(width: 18)
            VStack(alignment: .leading, spacing: 1) {
                Text(title).font(.caption.weight(.medium))
                Text(note).font(.system(size: 10)).foregroundStyle(.secondary).lineLimit(2)
            }
            Spacer(minLength: 6)
            Toggle("", isOn: isOn)
                .labelsHidden()
                .toggleStyle(.switch)
        }
    }

    private var ramNote: String {
        let settings = model.autoSettings.ram
        guard settings.enabled else { return "Off. Nothing is quit." }
        let limit = Formatting.compactBytes(settings.minAvailableBytes)
        let now = model.snapshot.map { Formatting.compactBytes($0.memory.availableBytes) } ?? "—"
        return "Asks apps you chose to quit if free RAM stays under \(limit). Now \(now) free."
    }

    private var storageNote: String {
        let settings = model.autoSettings.storage
        guard settings.enabled else { return "Off. Nothing is deleted." }
        return "Removes only the cache categories you ticked, every \(settings.intervalHours == 1 ? "hour" : "\(settings.intervalHours) hours")."
    }

    private func autoBinding(_ keyPath: WritableKeyPath<AutoCleanSettings, Bool>) -> Binding<Bool> {
        Binding(
            get: { model.autoSettings[keyPath: keyPath] },
            set: { on in
                var next = model.autoSettings
                next[keyPath: keyPath] = on
                Task { await model.saveAutoSettings(next) }
            }
        )
    }

    // MARK: Busiest apps

    @ViewBuilder
    private var appStrip: some View {
        if let apps = model.energy?.apps, !apps.isEmpty {
            VStack(alignment: .leading, spacing: 7) {
                HStack {
                    Text("Busiest apps").font(.caption.weight(.semibold))
                    Spacer()
                    Button("Energy") { openMainWindow(select: .energy) }
                        .buttonStyle(.plain)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
                HStack(spacing: 6) {
                    ForEach(Array(apps.prefix(6))) { app in
                        appTile(app)
                    }
                }
            }
        }
    }

    private func appTile(_ app: AppEnergy) -> some View {
        VStack(spacing: 3) {
            appIcon(app)
                .frame(width: 28, height: 28)
            Text(shortName(app.name))
                .font(.system(size: 9, weight: .medium))
                .lineLimit(1)
                .truncationMode(.tail)
            Text(Formatting.cpuPercent(app.cpuPercent))
                .font(.system(size: 10, weight: .semibold, design: .rounded))
                .monospacedDigit()
                .foregroundStyle(app.cpuPercent >= 80 ? CleanerTheme.coral : app.cpuPercent >= 30 ? CleanerTheme.amber : .secondary)
            Text(memoryLabel(app.memoryBytes))
                .font(.system(size: 9))
                .foregroundStyle(.tertiary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 6)
        .background(Color(nsColor: .controlBackgroundColor).opacity(0.96), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        .help("\(app.name): \(Formatting.cpuPercent(app.cpuPercent)) CPU, \(Formatting.bytes(app.memoryBytes)) memory, \(app.processCount) process(es)")
        .contextMenu {
            if app.canQuit {
                Button("Quit \(app.name)") { Task { await model.quitApp(app) } }
            }
            Button("Show in Energy") { openMainWindow(select: .energy) }
        }
    }

    private func shortName(_ name: String) -> String {
        name.replacingOccurrences(of: " Helper", with: "").replacingOccurrences(of: " (Renderer)", with: "")
    }

    /// "1.2 GB" or "320 MB", so a small app never reads as "0.0G".
    private func memoryLabel(_ bytes: UInt64) -> String {
        bytes >= 1_000_000_000
            ? String(format: "%.1f GB", Double(bytes) / 1_000_000_000)
            : String(format: "%.0f MB", Double(bytes) / 1_000_000)
    }

    @ViewBuilder
    private func appIcon(_ app: AppEnergy) -> some View {
        if let path = app.appPath {
            Image(nsImage: NSWorkspace.shared.icon(forFile: path)).resizable()
        } else {
            Image(systemName: "gearshape.2").resizable().scaledToFit().padding(5).foregroundStyle(.secondary)
        }
    }

    // MARK: Shortcuts

    private var shortcuts: some View {
        HStack(spacing: 6) {
            shortcut("macwindow", "Open", "Open Mac Cleaner") { openMainWindow(select: nil) }
            shortcut("trash", "Clean", "Free Up Space") { openMainWindow(select: .cleanup) }
            shortcut("battery.50percent", "Energy", "What drains CPU and battery") { openMainWindow(select: .energy) }
            shortcut("archivebox", "Recovery", "Items waiting to be erased or restored") { openMainWindow(select: .quarantine) }
            shortcut("flag.fill", "Checkpoint", "Set a checkpoint now") { Task { await model.setStorageBaseline() } }
            shortcut("arrow.clockwise", "Refresh", "Refresh status") { Task { await model.refreshStatus() } }
        }
    }

    private func shortcut(_ symbol: String, _ caption: String, _ help: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            VStack(spacing: 3) {
                Image(systemName: symbol).font(.system(size: 14, weight: .medium))
                Text(caption).font(.system(size: 9)).lineLimit(1).minimumScaleFactor(0.8)
            }
            .frame(maxWidth: .infinity, minHeight: 36)
        }
        .buttonStyle(.bordered)
        .help(help)
        .accessibilityLabel(help)
        .disabled(model.isBusy && caption != "Open")
    }

    // MARK: Footer

    private var footer: some View {
        HStack {
            Text("Local only · nothing runs without your say")
                .font(.system(size: 10))
                .foregroundStyle(.secondary)
            Spacer()
            Button("Quit") { NSApp.terminate(nil) }
                .buttonStyle(.plain)
                .font(.caption)
                .foregroundStyle(.primary.opacity(0.72))
                .keyboardShortcut("q")
        }
    }

    // MARK: Helpers

    private var statusText: String {
        guard let snapshot = model.snapshot else { return "Waiting" }
        if snapshot.cpu.pressure != "normal" { return "\(snapshot.cpu.pressure.capitalized) CPU" }
        if snapshot.memory.pressure != "normal" { return "\(snapshot.memory.pressure.capitalized) memory" }
        return "Healthy"
    }

    private var statusColor: Color {
        guard let snapshot = model.snapshot else { return .secondary }
        if snapshot.cpu.pressure != "normal" { return cpuTint(snapshot.cpu.pressure) }
        return memoryTint(snapshot.memory.pressure)
    }

    private func openMainWindow(select item: SidebarItem?) {
        if let item { model.selection = item }
        if let window = NSApp.windows.first(where: { $0.canBecomeKey && !($0 is NSPanel) }) {
            window.makeKeyAndOrderFront(nil)
        } else {
            openWindow(id: "main")
        }
        NSApp.activate(ignoringOtherApps: true)
    }

    private func memoryTint(_ pressure: String) -> Color {
        switch pressure {
        case "critical": .red
        case "warning": .orange
        default: .green
        }
    }

    private func storageTint(_ available: UInt64, total: UInt64) -> Color {
        let fraction = Formatting.percent(available, of: total)
        if fraction < 0.05 { return .red }
        if fraction < 0.15 { return .orange }
        return CleanerTheme.mint
    }

    private func cpuTint(_ pressure: String) -> Color {
        switch pressure {
        case "critical": CleanerTheme.coral
        case "warning": CleanerTheme.amber
        default: CleanerTheme.azure
        }
    }
}

/// A small coloured dot followed by the title.
private struct StatusDotStyle: LabelStyle {
    let tint: Color

    func makeBody(configuration: Configuration) -> some View {
        HStack(spacing: 5) {
            configuration.icon.font(.system(size: 6)).foregroundStyle(tint)
            configuration.title
        }
    }
}
