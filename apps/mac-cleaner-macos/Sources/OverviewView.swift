import Foundation
import SwiftUI

struct OverviewView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                header
                if let snapshot = model.snapshot {
                    healthGrid(snapshot)
                    SpaceChangeCard()
                    quickWins(snapshot)
                    ProcessPanel(
                        processes: highlightedProcesses(snapshot.processes),
                        onInspect: { model.openApplication("Activity Monitor") }
                    )
                } else {
                    NativeEmptyState("Loading system status", systemImage: "gauge.with.dots.needle.50percent")
                        .frame(minHeight: 420)
                }
            }
            .padding(.horizontal, 26)
            .padding(.top, 22)
            .padding(.bottom, 30)
        }
        .navigationTitle("Overview")
    }

    private var header: some View {
        HStack(alignment: .bottom, spacing: 20) {
            VStack(alignment: .leading, spacing: 6) {
                Text("SYSTEM OVERVIEW")
                    .font(.system(size: 10, weight: .bold, design: .rounded))
                    .tracking(1.8)
                    .foregroundStyle(CleanerTheme.coral)
                Text("Your Mac, at a glance")
                    .font(.system(size: 31, weight: .bold, design: .rounded))
                    .tracking(-0.7)
                Text("Live, read-only health signals. Developer work stays protected.")
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            if let pressure = model.snapshot?.memory.pressure {
                PressureBadge(pressure: pressure)
            }
            Button {
                model.openApplication("Activity Monitor")
            } label: {
                Label("Activity Monitor", systemImage: "waveform.path.ecg")
            }
            .buttonStyle(.bordered)
            .controlSize(.large)
        }
    }

    private func healthGrid(_ snapshot: SystemSnapshot) -> some View {
        HStack(spacing: 14) {
            StorageSpotlight(disk: snapshot.disk)
                .frame(maxWidth: .infinity)

            VStack(spacing: 8) {
                CompactHealthCard(
                    eyebrow: "MEMORY AVAILABLE",
                    value: Formatting.bytes(snapshot.memory.availableBytes),
                    detail: "\(Formatting.bytes(snapshot.memory.compressedBytes)) compressed",
                    symbol: "memorychip.fill",
                    tint: snapshot.memory.pressure == "normal" ? CleanerTheme.azure : CleanerTheme.amber
                )
                CompactHealthCard(
                    eyebrow: "SWAP IN USE",
                    value: Formatting.bytes(snapshot.memory.swapUsedBytes),
                    detail: snapshot.memory.pressure.capitalized + " memory pressure",
                    symbol: "arrow.left.arrow.right",
                    tint: snapshot.memory.pressure == "critical" ? CleanerTheme.coral : CleanerTheme.violet
                )
                CompactHealthCard(
                    eyebrow: "CPU LOAD",
                    value: Formatting.cpuPercent(snapshot.cpu.usagePercent),
                    detail: "Whole Mac · \(snapshot.cpu.logicalCores) logical cores · \(snapshot.cpu.pressure.capitalized)",
                    symbol: "cpu",
                    tint: cpuTint(snapshot.cpu.pressure)
                )
            }
            .frame(width: 262)
        }
    }

    private func quickWins(_ snapshot: SystemSnapshot) -> some View {
        let topMemory = snapshot.processes.max { $0.memoryBytes < $1.memoryBytes }
        let topCPU = snapshot.processes.max { $0.cpuPercent < $1.cpuPercent }
        let candidateBytes = model.scan?.candidates.reduce(UInt64(0)) { $0 &+ $1.bytes } ?? 0
        let reviewableBytes = model.storage?.reviewableBytes ?? 0

        return VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .firstTextBaseline) {
                VStack(alignment: .leading, spacing: 3) {
                    Text("Quick wins")
                        .font(.system(size: 20, weight: .semibold, design: .rounded))
                    Text("Start with reversible cleanup and the processes creating pressure right now.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Text("NOTHING RUNS WITHOUT REVIEW")
                    .font(.system(size: 8, weight: .bold, design: .rounded))
                    .tracking(1.0)
                    .foregroundStyle(.secondary)
            }

            QuickWinRow(
                symbol: "internaldrive.fill",
                title: cleanupTitle(candidateBytes: candidateBytes, reviewableBytes: reviewableBytes),
                detail: cleanupDetail(candidateBytes: candidateBytes, reviewableBytes: reviewableBytes),
                actionTitle: candidateBytes > 0 || reviewableBytes > 0 ? "Free up space" : "Scan",
                tint: CleanerTheme.mint,
                enabled: !model.isBusy
            ) {
                model.selection = .cleanup
                if model.storage == nil || model.scan == nil {
                    Task {
                        await model.analyzeStorage()
                        await model.scanCleanup()
                    }
                }
            }

            if let topMemory {
                QuickWinRow(
                    symbol: "memorychip.fill",
                    title: snapshot.memory.pressure == "normal" ? "See what holds memory" : "Relieve \(snapshot.memory.pressure) memory pressure",
                    detail: "\(topMemory.name) leads at \(Formatting.bytes(topMemory.memoryBytes)); swap is \(Formatting.bytes(snapshot.memory.swapUsedBytes)). Inspect before quitting anything.",
                    actionTitle: "Inspect memory",
                    tint: snapshot.memory.pressure == "critical" ? CleanerTheme.coral : CleanerTheme.amber
                ) {
                    model.openApplication("Activity Monitor")
                }
            }

            if let topCPU {
                QuickWinRow(
                    symbol: "cpu",
                    title: "Investigate \(topCPU.name)’s CPU use",
                    detail: "\(Formatting.cpuPercent(topCPU.cpuPercent)) equals about \(String(format: "%.1f", topCPU.cpuPercent / 100)) logical cores. Developer processes stay protected.",
                    actionTitle: "Inspect CPU",
                    tint: topCPU.cpuPercent >= 100 ? CleanerTheme.coral : CleanerTheme.azure
                ) {
                    model.openApplication("Activity Monitor")
                }
            }
        }
        .padding(18)
        .background(Color.primary.opacity(0.028), in: RoundedRectangle(cornerRadius: 20, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 20, style: .continuous)
                .stroke(Color.primary.opacity(0.08), lineWidth: 1)
        }
    }

    private func cleanupTitle(candidateBytes: UInt64, reviewableBytes: UInt64) -> String {
        if candidateBytes > 0 { return "\(Formatting.bytes(candidateBytes)) can be freed" }
        if reviewableBytes > 0 { return "Review \(Formatting.bytes(reviewableBytes)) of cache and generated data" }
        return "Find disposable caches and old build output"
    }

    private func cleanupDetail(candidateBytes: UInt64, reviewableBytes: UInt64) -> String {
        if candidateBytes > 0 {
            return "Caches and generated project folders. Pick what to remove; nothing is deleted without your confirmation."
        }
        if reviewableBytes > 0 {
            return "Measured reviewable data can include caches, recordings, and generated builds—not repositories or SDKs."
        }
        return "Scan verified simulator recordings, uv downloads, and old generated build data."
    }

    private func highlightedProcesses(_ processes: [ProcessSnapshot]) -> [ProcessSnapshot] {
        let memory = processes.sorted { $0.memoryBytes > $1.memoryBytes }.prefix(8)
        let cpu = processes.sorted { $0.cpuPercent > $1.cpuPercent }.prefix(8)
        var seen = Set<String>()
        return (Array(memory) + Array(cpu)).filter { seen.insert($0.id).inserted }
    }

    private func cpuTint(_ pressure: String) -> Color {
        switch pressure {
        case "critical": CleanerTheme.coral
        case "warning": CleanerTheme.amber
        default: CleanerTheme.azure
        }
    }
}

private struct StorageSpotlight: View {
    let disk: DiskSnapshot

    private var usedBytes: UInt64 { disk.totalBytes >= disk.availableBytes ? disk.totalBytes - disk.availableBytes : 0 }
    private var usedFraction: Double { Formatting.percent(usedBytes, of: disk.totalBytes) }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 5) {
                    Label("STORAGE HEADROOM", systemImage: "internaldrive.fill")
                        .font(.system(size: 10, weight: .bold, design: .rounded))
                        .tracking(1.25)
                        .foregroundStyle(CleanerTheme.mint)
                    Text("Plenty of room to build")
                        .font(.system(size: 15, weight: .semibold, design: .rounded))
                }
                Spacer()
                ZStack {
                    Circle().fill(CleanerTheme.mint.opacity(0.14))
                    Image(systemName: "internaldrive.fill")
                        .font(.system(size: 16, weight: .semibold))
                        .foregroundStyle(CleanerTheme.mint)
                }
                .frame(width: 34, height: 34)
            }

            Spacer(minLength: 6)

            HStack(alignment: .firstTextBaseline, spacing: 7) {
                Text(Formatting.bytes(disk.availableBytes))
                    .font(.system(size: 30, weight: .bold, design: .rounded))
                    .tracking(-1.25)
                    .monospacedDigit()
                Text("free")
                    .font(.headline)
                    .foregroundStyle(.secondary)
            }

            Spacer(minLength: 8)

            GeometryReader { geometry in
                ZStack(alignment: .leading) {
                    Capsule().fill(Color.primary.opacity(0.09))
                    Capsule()
                        .fill(LinearGradient(colors: [CleanerTheme.azure, CleanerTheme.mint], startPoint: .leading, endPoint: .trailing))
                        .frame(width: max(8, geometry.size.width * usedFraction))
                }
            }
            .frame(height: 7)
            .accessibilityLabel("Storage used")
            .accessibilityValue(Formatting.bytes(usedBytes))

            HStack {
                Text("\(Formatting.bytes(usedBytes)) used")
                Spacer()
                Text("\(Formatting.bytes(disk.totalBytes)) total")
            }
            .font(.caption2.monospacedDigit())
            .foregroundStyle(.secondary)
            .padding(.top, 7)
        }
        .padding(14)
        .background(
            LinearGradient(
                colors: [CleanerTheme.mint.opacity(0.13), CleanerTheme.azure.opacity(0.075), Color.primary.opacity(0.025)],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            ),
            in: RoundedRectangle(cornerRadius: 22, style: .continuous)
        )
        .overlay {
            RoundedRectangle(cornerRadius: 22, style: .continuous)
                .stroke(LinearGradient(colors: [CleanerTheme.mint.opacity(0.32), Color.primary.opacity(0.07)], startPoint: .topLeading, endPoint: .bottomTrailing), lineWidth: 1)
        }
        .shadow(color: CleanerTheme.mint.opacity(0.08), radius: 24, y: 12)
    }
}

private struct CompactHealthCard: View {
    let eyebrow: String
    let value: String
    let detail: String
    let symbol: String
    let tint: Color

    var body: some View {
        HStack(spacing: 14) {
            ZStack {
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .fill(tint.opacity(0.15))
                Image(systemName: symbol)
                    .font(.system(size: 17, weight: .semibold))
                    .foregroundStyle(tint)
            }
            .frame(width: 34, height: 34)

            VStack(alignment: .leading, spacing: 1) {
                Text(eyebrow)
                    .font(.system(size: 9, weight: .bold, design: .rounded))
                    .tracking(1.0)
                    .foregroundStyle(tint)
                Text(value)
                    .font(.system(size: 17, weight: .bold, design: .rounded))
                    .tracking(-0.35)
                    .monospacedDigit()
                Text(detail)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .frame(maxWidth: .infinity, minHeight: 62, alignment: .leading)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 18, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 18, style: .continuous)
                .stroke(tint.opacity(0.18), lineWidth: 1)
        }
    }
}

private struct PressureBadge: View {
    let pressure: String

    private var tint: Color {
        switch pressure {
        case "critical": CleanerTheme.coral
        case "warning": CleanerTheme.amber
        default: CleanerTheme.mint
        }
    }

    var body: some View {
        HStack(spacing: 7) {
            Circle()
                .fill(tint)
                .frame(width: 7, height: 7)
                .shadow(color: tint.opacity(0.7), radius: 5)
            Text(pressure == "normal" ? "Healthy" : "\(pressure.capitalized) pressure")
                .font(.caption.weight(.semibold))
        }
        .foregroundStyle(tint)
        .padding(.horizontal, 11)
        .padding(.vertical, 7)
        .background(tint.opacity(0.11), in: Capsule())
        .overlay { Capsule().stroke(tint.opacity(0.22), lineWidth: 1) }
    }
}

private struct ProcessPanel: View {
    let processes: [ProcessSnapshot]
    let onInspect: () -> Void

    private var maximumMemory: UInt64 { processes.map(\.memoryBytes).max() ?? 1 }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(alignment: .bottom, spacing: 13) {
                VStack(alignment: .leading, spacing: 3) {
                    Text("What’s active")
                        .font(.system(size: 18, weight: .semibold, design: .rounded))
                    Text("Largest memory and CPU consumers right now")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Text("MEMORY")
                    .font(.system(size: 9, weight: .bold, design: .rounded))
                    .tracking(1.05)
                    .foregroundStyle(.tertiary)
                    .frame(width: 110, alignment: .trailing)
                VStack(alignment: .trailing, spacing: 1) {
                    Text("CPU %")
                    Text("100% = 1 CORE")
                        .font(.system(size: 7, weight: .semibold, design: .rounded))
                }
                .font(.system(size: 9, weight: .bold, design: .rounded))
                .tracking(0.8)
                .foregroundStyle(.tertiary)
                .frame(width: 78, alignment: .trailing)
                Text("ACTION")
                    .font(.system(size: 9, weight: .bold, design: .rounded))
                    .tracking(1.05)
                    .foregroundStyle(.tertiary)
                    .frame(width: 92)
            }
            .padding(.horizontal, 18)
            .padding(.vertical, 15)

            Divider().opacity(0.55)

            LazyVStack(spacing: 1) {
                ForEach(processes) { process in
                    ProcessRow(process: process, maximumMemory: maximumMemory, onInspect: onInspect)
                }
            }
            .padding(6)
        }
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 20, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 20, style: .continuous)
                .stroke(Color.primary.opacity(0.08), lineWidth: 1)
        }
    }
}

private struct ProcessRow: View {
    let process: ProcessSnapshot
    let maximumMemory: UInt64
    let onInspect: () -> Void
    @State private var isHovering = false

    private var tint: Color {
        let palette = [CleanerTheme.azure, CleanerTheme.mint, CleanerTheme.violet, CleanerTheme.amber, CleanerTheme.coral]
        return palette[Int(process.pid) % palette.count]
    }

    var body: some View {
        HStack(spacing: 13) {
            ZStack {
                RoundedRectangle(cornerRadius: 9, style: .continuous)
                    .fill(tint.opacity(0.13))
                Image(systemName: process.protected ? "hammer.fill" : "app.dashed")
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(tint)
            }
            .frame(width: 34, height: 34)

            VStack(alignment: .leading, spacing: 2) {
                Text(process.name)
                    .font(.system(size: 13, weight: .semibold, design: .rounded))
                    .lineLimit(1)
                Text("PID \(process.pid) · \(process.protected ? "developer work protected" : "reviewable app")")
                    .font(.caption2.monospacedDigit())
                    .foregroundStyle(.tertiary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)

            VStack(alignment: .trailing, spacing: 5) {
                Text(Formatting.bytes(process.memoryBytes))
                    .font(.caption.weight(.semibold).monospacedDigit())
                GeometryReader { geometry in
                    ZStack(alignment: .leading) {
                        Capsule().fill(Color.primary.opacity(0.07))
                        Capsule().fill(tint.opacity(0.7))
                            .frame(width: max(4, geometry.size.width * Double(process.memoryBytes) / Double(maximumMemory)))
                    }
                }
                .frame(height: 3)
            }
            .frame(width: 110)

            Text("\(process.cpuPercent, format: .number.precision(.fractionLength(1)))%")
                .font(.caption.monospacedDigit())
                .foregroundStyle(.secondary)
                .frame(width: 78, alignment: .trailing)
                .help("Per-process CPU. 100% equals one fully used logical core, so multi-threaded processes can exceed 100%.")

            Button(action: onInspect) {
                HStack(spacing: 6) {
                    Image(systemName: "arrow.up.forward.app")
                        .font(.system(size: 9, weight: .bold))
                    Text("Inspect")
                        .font(.caption2.weight(.semibold))
                }
                .foregroundStyle(process.protected ? CleanerTheme.mint : .secondary)
                .padding(.horizontal, 9)
                .padding(.vertical, 5)
                .background((process.protected ? CleanerTheme.mint : Color.secondary).opacity(0.09), in: Capsule())
            }
            .buttonStyle(.plain)
            .frame(width: 92)
            .help(process.protected ? "Open Activity Monitor. Mac Cleaner will not quit this developer process." : "Open Activity Monitor to review this process.")
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 7)
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(isHovering ? Color.primary.opacity(0.055) : .clear)
        )
        .contentShape(Rectangle())
        .onHover { isHovering = $0 }
    }
}

private struct QuickWinRow: View {
    let symbol: String
    let title: String
    let detail: String
    let actionTitle: String
    let tint: Color
    var enabled = true
    let action: () -> Void

    var body: some View {
        HStack(spacing: 13) {
            ZStack {
                RoundedRectangle(cornerRadius: 11, style: .continuous)
                    .fill(tint.opacity(0.14))
                Image(systemName: symbol)
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundStyle(tint)
            }
            .frame(width: 42, height: 42)

            VStack(alignment: .leading, spacing: 3) {
                Text(title)
                    .font(.system(size: 14, weight: .semibold, design: .rounded))
                Text(detail)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }

            Spacer(minLength: 12)

            Button(actionTitle, action: action)
                .buttonStyle(.bordered)
                .tint(tint)
                .controlSize(.regular)
                .frame(width: 124)
                .disabled(!enabled)
        }
        .padding(12)
        .background(Color.primary.opacity(0.035), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
    }
}

// MARK: Where did my space go

/// Answers "where did my free space go since the last checkpoint?" from measured data only:
/// the whole-disk change, each tracked location's change, and an honest unexplained remainder.
private struct SpaceChangeCard: View {
    @EnvironmentObject private var model: AppModel

    private let minimumShown: Int64 = 50_000_000

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            header
            if let storage = model.storage {
                if model.justCheckpointed {
                    justSet(storage)
                } else if let comparison = storage.comparison, comparison.kind == "baseline" {
                    changes(storage, comparison)
                } else {
                    noCheckpoint
                }
            } else {
                HStack(spacing: 10) {
                    ProgressView().controlSize(.small)
                    Text("Measuring known storage locations…").font(.callout).foregroundStyle(.secondary)
                }
            }
        }
        .padding(18)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 20, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 20, style: .continuous)
                .stroke(CleanerTheme.amber.opacity(0.22), lineWidth: 1)
        }
    }

    private var header: some View {
        HStack(alignment: .top) {
            VStack(alignment: .leading, spacing: 4) {
                Text("WHERE DID MY SPACE GO")
                    .font(.system(size: 10, weight: .bold, design: .rounded))
                    .tracking(1.4)
                    .foregroundStyle(CleanerTheme.amber)
                Text("Changes since your last checkpoint")
                    .font(.system(size: 20, weight: .semibold, design: .rounded))
            }
            Spacer()
            Button("Re-measure", systemImage: "arrow.clockwise") { Task { await model.analyzeStorage() } }
                .buttonStyle(.bordered)
                .disabled(model.isBusy)
            Button("Set checkpoint now", systemImage: "flag.fill") { Task { await model.setStorageBaseline() } }
                .buttonStyle(.bordered)
                .tint(CleanerTheme.amber)
                .disabled(model.isBusy || model.storage == nil)
        }
    }

    private var noCheckpoint: some View {
        Text("No checkpoint yet. Clean something up or press “Set checkpoint now”. From then on, this box lists every place your free space went, with sizes.")
            .font(.callout)
            .foregroundStyle(.secondary)
    }

    private func justSet(_ storage: StorageAnalysis) -> some View {
        Text("Checkpoint saved at \(Formatting.bytes(storage.availableBytes)) free. Use your Mac as normal, then press Re-measure to see exactly where space went.")
            .font(.callout)
            .foregroundStyle(.secondary)
    }

    private func changes(_ storage: StorageAnalysis, _ comparison: StorageComparison) -> some View {
        // The itemised list comes from the last measurement; the headline uses live free space
        // so it never lags behind the Storage card above.
        let measuredFree: UInt64? = storage.availableBytes
        let checkpointFree = Int64(clamping: storage.availableBytes) &+ comparison.volumeUsedDeltaBytes
        let liveFree = Int64(clamping: model.snapshot?.disk.availableBytes ?? storage.availableBytes)
        let freeChange = liveFree &- checkpointFree
        // Swap is read live, so its row never lags behind the headline. Everything else is as of the last measurement.
        let swapID = "system-memory-swap"
        let swapSpot = storage.hotspots.first { $0.id == swapID }
        let liveSwap = Int64(clamping: model.snapshot?.memory.swapUsedBytes ?? swapSpot?.bytes ?? 0)
        let swapDeltaLive: Int64? = swapSpot?.previousBytes.map { liveSwap &- Int64(clamping: $0) }
        let movers = storage.hotspots
            .compactMap { spot -> (StorageHotspot, Int64)? in
                guard spot.id != swapID, let delta = spot.deltaBytes, abs(delta) >= minimumShown else { return nil }
                return (spot, delta)
            }
            .sorted { abs($0.1) > abs($1.1) }
            .prefix(6)
        // Free space also moved after the last measurement. Swap's part of that is known live;
        // the rest happened somewhere that only a fresh Re-measure can name.
        let sinceMeasurement = measuredFree.map { Int64(clamping: $0) &- liveFree } ?? 0
        let swapSinceMeasurement = swapSpot.map { liveSwap &- Int64(clamping: $0.bytes) } ?? 0
        let otherSinceMeasurement = sinceMeasurement &- swapSinceMeasurement
        // What the measured locations could not account for at the moment of the last measurement.
        let unexplained = comparison.accountingGapBytes
        // Items the checkpoint never measured: their growth cannot be split out, so it lands in the remainder.
        let newlyTracked = storage.hotspots
            .filter { $0.comparisonState == "newly-tracked" && $0.bytes >= UInt64(minimumShown) }
            .sorted { $0.bytes > $1.bytes }
        let newlyTrackedBytes = newlyTracked.reduce(UInt64(0)) { $0 &+ $1.bytes }
        return VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(Formatting.signedBytes(freeChange))
                    .font(.system(size: 30, weight: .bold, design: .rounded))
                    .foregroundStyle(freeChange < 0 ? CleanerTheme.coral : CleanerTheme.mint)
                    .monospacedDigit()
                Text("free space since your checkpoint")
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }
            HStack(spacing: 8) {
                Image(systemName: "flag.fill").foregroundStyle(CleanerTheme.amber)
                Text("Checkpoint: \(Formatting.dateTime(comparison.capturedAt)) (\(Formatting.relative(comparison.capturedAt))) · \(Formatting.bytes(UInt64(clamping: checkpointFree))) free then, \(Formatting.bytes(UInt64(clamping: liveFree))) free now")
                    .font(.callout.weight(.medium))
            }
            if movers.isEmpty {
                Text("No tracked location changed by more than 50 MB.")
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }
            VStack(spacing: 1) {
                if let swapSpot, let delta = swapDeltaLive, abs(delta) >= minimumShown {
                    row(
                        title: swapSpot.displayName,
                        note: delta > 0
                            ? "Live. Temporary: macOS writes memory to disk under pressure and gives it back later."
                            : "Live. macOS released swap, so this space came back.",
                        delta: delta,
                        tint: delta > 0 ? CleanerTheme.coral : CleanerTheme.mint
                    )
                }
                ForEach(Array(movers), id: \.0.id) { spot, delta in
                    moverRow(spot, delta)
                }
                if abs(otherSinceMeasurement) >= minimumShown {
                    row(
                        title: "Changed since the last measurement",
                        note: "Happened after \(Formatting.relative(storage.capturedAt)). Press Re-measure to name the exact locations.",
                        delta: otherSinceMeasurement,
                        tint: CleanerTheme.violet
                    )
                }
                if abs(unexplained) >= minimumShown {
                    row(
                        title: "Still unexplained",
                        note: newlyTrackedBytes >= UInt64(max(unexplained, 0)) / 2
                            ? "Mostly growth of the newly tracked items below, which were never measured at the checkpoint."
                            : "Not in any location Mac Cleaner tracks. Set a new checkpoint so growth in the newly tracked items below is itemised next time.",
                        delta: unexplained,
                        tint: CleanerTheme.violet
                    )
                }
            }
            Text("Per-location sizes measured \(Formatting.relative(storage.capturedAt)). Press Re-measure to refresh them; the headline above is live.")
                .font(.caption2)
                .foregroundStyle(.tertiary)
            if !newlyTracked.isEmpty {
                VStack(alignment: .leading, spacing: 1) {
                    Text("NEWLY TRACKED · size now, change unknown until the next checkpoint")
                        .font(.system(size: 9, weight: .bold, design: .rounded))
                        .tracking(0.8)
                        .foregroundStyle(.tertiary)
                        .padding(.bottom, 4)
                    ForEach(newlyTracked) { spot in
                        row(
                            title: spot.displayName,
                            note: note(for: spot, delta: 0),
                            text: Formatting.bytes(spot.id == "system-memory-swap" ? (model.snapshot?.memory.swapUsedBytes ?? spot.bytes) : spot.bytes) + " now",
                            tint: CleanerTheme.violet
                        )
                    }
                }
            }
        }
    }

    private func moverRow(_ spot: StorageHotspot, _ delta: Int64) -> some View {
        row(
            title: spot.displayName,
            note: note(for: spot, delta: delta),
            delta: delta,
            tint: delta > 0 ? CleanerTheme.coral : CleanerTheme.mint
        )
    }

    private func note(for spot: StorageHotspot, delta: Int64) -> String {
        if spot.id == "system-memory-swap" {
            return "Temporary. macOS writes memory to disk under pressure and gives it back later."
        }
        switch spot.disposition {
        case "reviewable": return "Rebuildable. Free Up Space can remove it."
        case "tool-managed": return "Owned by another app. Clean it from that app."
        default: return "Protected. Mac Cleaner only reports it."
        }
    }

    private func row(title: String, note: String, delta: Int64, tint: Color) -> some View {
        row(title: title, note: note, text: Formatting.signedBytes(delta), tint: tint)
    }

    private func row(title: String, note: String, text: String, tint: Color) -> some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                Text(title).font(.callout.weight(.semibold)).lineLimit(1)
                Text(note).font(.caption2).foregroundStyle(.secondary).lineLimit(1)
            }
            Spacer()
            Text(text)
                .font(.callout.weight(.semibold).monospacedDigit())
                .foregroundStyle(tint)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 7)
        .background(Color.primary.opacity(0.035), in: RoundedRectangle(cornerRadius: 11, style: .continuous))
    }
}
