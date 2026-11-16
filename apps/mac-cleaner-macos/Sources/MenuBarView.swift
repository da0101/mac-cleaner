import AppKit
import SwiftUI

struct MenuBarView: View {
    @Environment(\.openWindow) private var openWindow
    @EnvironmentObject private var model: AppModel

    var body: some View {
        VStack(spacing: 14) {
            header

            if let snapshot = model.snapshot {
                HStack(spacing: 10) {
                    MetricCard(
                        title: "CPU",
                        value: Formatting.cpuPercent(snapshot.cpu.usagePercent),
                        detail: "\(snapshot.cpu.logicalCores) logical cores",
                        symbol: "cpu",
                        fraction: min(snapshot.cpu.usagePercent / 100, 1),
                        tint: cpuTint(snapshot.cpu.pressure)
                    )
                    MetricCard(
                        title: "Memory",
                        value: Formatting.bytes(snapshot.memory.availableBytes),
                        detail: "available",
                        symbol: "memorychip",
                        fraction: Formatting.percent(snapshot.memory.availableBytes, of: snapshot.memory.totalBytes),
                        tint: memoryTint(snapshot.memory.pressure)
                    )
                    MetricCard(
                        title: "Storage",
                        value: Formatting.bytes(snapshot.disk.availableBytes),
                        detail: "free",
                        symbol: "internaldrive",
                        fraction: Formatting.percent(snapshot.disk.availableBytes, of: snapshot.disk.totalBytes),
                        tint: storageTint(snapshot.disk.availableBytes, total: snapshot.disk.totalBytes)
                    )
                }

                cpuConsumers(snapshot.processes)
            } else {
                HStack(spacing: 10) {
                    ProgressView()
                        .controlSize(.small)
                    Text("Reading system status…")
                        .foregroundStyle(.secondary)
                    Spacer()
                }
                .padding(14)
                .background(.quaternary.opacity(0.55), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            }

            VStack(spacing: 8) {
                Button(action: openMainWindow) {
                    Label("Open Mac Cleaner", systemImage: "macwindow")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .tint(CleanerTheme.coral)
                .controlSize(.large)

                HStack(spacing: 8) {
                    Button {
                        openMainWindow()
                        model.selection = .storage
                        Task { await model.analyzeStorage() }
                    } label: {
                        Label("Analyze Storage", systemImage: "magnifyingglass")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.bordered)
                    .tint(CleanerTheme.azure)
                    .disabled(model.isBusy)

                    Button {
                        Task { await model.refreshStatus() }
                    } label: {
                        Image(systemName: "arrow.clockwise")
                            .frame(width: 18)
                    }
                    .buttonStyle(.bordered)
                    .help("Refresh status")
                    .disabled(model.isBusy)
                }
            }

            Divider()

            HStack {
                Label(statusText, systemImage: statusSymbol)
                    .font(.caption)
                    .foregroundStyle(.primary.opacity(0.72))
                Spacer()
                Button("Quit") { NSApp.terminate(nil) }
                    .buttonStyle(.plain)
                    .foregroundStyle(.primary.opacity(0.72))
                    .keyboardShortcut("q")
            }
        }
        .padding(16)
        .frame(width: 430)
        .background(Color(nsColor: .windowBackgroundColor))
        .task { model.startMonitoring() }
    }

    private var header: some View {
        HStack(spacing: 11) {
            CleanerBrandMark()
                .frame(width: 44, height: 44)
                .accessibilityHidden(true)

            VStack(alignment: .leading, spacing: 2) {
                Text("Mac Cleaner")
                    .font(.headline)
                Text("Developer storage, explained")
                    .font(.caption)
                    .foregroundStyle(.primary.opacity(0.68))
            }

            Spacer()

            if model.isBusy {
                ProgressView()
                    .controlSize(.small)
                    .accessibilityLabel("Mac Cleaner is working")
            } else {
                Circle()
                    .fill(statusColor)
                    .frame(width: 8, height: 8)
                    .accessibilityLabel(statusText)
            }
        }
    }

    private var statusText: String {
        guard let snapshot = model.snapshot else { return "Waiting for status" }
        if snapshot.cpu.pressure != "normal" {
            return "\(snapshot.cpu.pressure.capitalized) CPU load"
        }
        if snapshot.memory.pressure != "normal" {
            return "\(snapshot.memory.pressure.capitalized) memory pressure"
        }
        return "Monitoring locally · nothing automatic"
    }

    private var statusSymbol: String {
        guard let snapshot = model.snapshot else { return "clock" }
        return snapshot.memory.pressure == "normal" && snapshot.cpu.pressure == "normal"
            ? "checkmark.shield"
            : "exclamationmark.triangle"
    }

    private var statusColor: Color {
        guard let snapshot = model.snapshot else { return .secondary }
        if snapshot.cpu.pressure != "normal" { return cpuTint(snapshot.cpu.pressure) }
        return memoryTint(snapshot.memory.pressure)
    }

    private func openMainWindow() {
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

    @ViewBuilder
    private func cpuConsumers(_ processes: [ProcessSnapshot]) -> some View {
        let active = Array(processes.filter { $0.cpuPercent >= 0.5 }.sorted { $0.cpuPercent > $1.cpuPercent }.prefix(4))
        if !active.isEmpty {
            VStack(alignment: .leading, spacing: 7) {
                HStack {
                    Label("Top CPU right now", systemImage: "waveform.path.ecg")
                        .font(.caption.weight(.semibold))
                    Spacer()
                    Text("READ-ONLY")
                        .font(.system(size: 8, weight: .bold, design: .rounded))
                        .tracking(1)
                        .foregroundStyle(.primary.opacity(0.56))
                }
                ForEach(active) { process in
                    HStack(spacing: 8) {
                        Circle()
                            .fill(process.protected ? CleanerTheme.mint : CleanerTheme.azure)
                            .frame(width: 6, height: 6)
                        VStack(alignment: .leading, spacing: 1) {
                            Text(process.name)
                                .font(.caption.weight(.medium))
                                .lineLimit(1)
                            Text("PID \(process.pid)\(process.protected ? " · developer work protected" : "")")
                                .font(.caption2)
                                .foregroundStyle(.primary.opacity(0.64))
                        }
                        Spacer()
                        Text(process.cpuPercent, format: .number.precision(.fractionLength(1)))
                            .font(.caption.weight(.semibold).monospacedDigit())
                            .foregroundStyle(processTint(process.cpuPercent))
                        Text("%")
                            .font(.caption2)
                            .foregroundStyle(.primary.opacity(0.68))
                    }
                }
            }
            .padding(12)
            .background(
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .fill(Color(nsColor: .controlBackgroundColor).opacity(0.96))
            )
            .overlay {
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .stroke(Color.primary.opacity(0.10), lineWidth: 1)
            }
        }
    }

    private func processTint(_ percent: Double) -> Color {
        if percent >= 100 { return .red }
        if percent >= 50 { return CleanerTheme.amber }
        return CleanerTheme.azure
    }
}

private struct MetricCard: View {
    let title: String
    let value: String
    let detail: String
    let symbol: String
    let fraction: Double
    let tint: Color

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 5) {
                Image(systemName: symbol)
                    .foregroundStyle(tint)
                Text(title)
                    .foregroundStyle(.primary.opacity(0.72))
            }
            .font(.caption.weight(.semibold))

            VStack(alignment: .leading, spacing: 1) {
                Text(value)
                    .font(.system(.title3, design: .rounded, weight: .semibold))
                    .foregroundStyle(tint)
                    .monospacedDigit()
                    .lineLimit(1)
                    .minimumScaleFactor(0.8)
                Text(detail)
                    .font(.caption2)
                    .foregroundStyle(.primary.opacity(0.68))
            }

            GeometryReader { proxy in
                ZStack(alignment: .leading) {
                    Capsule()
                        .fill(Color.primary.opacity(0.12))
                    Capsule()
                        .fill(tint.gradient)
                        .frame(width: max(5, proxy.size.width * min(max(fraction, 0), 1)))
                }
            }
            .frame(height: 7)
                .accessibilityLabel("\(title) \(detail)")
                .accessibilityValue(value)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(Color(nsColor: .controlBackgroundColor).opacity(0.96))
        )
        .overlay {
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .stroke(tint.opacity(0.24), lineWidth: 1)
        }
    }
}
