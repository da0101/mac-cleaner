import SwiftUI

struct OverviewView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                header
                if let snapshot = model.snapshot {
                    healthGrid(snapshot)
                    ProcessPanel(processes: highlightedProcesses(snapshot.processes))
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

            VStack(spacing: 12) {
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
                    detail: "\(snapshot.cpu.logicalCores) logical cores · \(snapshot.cpu.pressure.capitalized)",
                    symbol: "cpu",
                    tint: cpuTint(snapshot.cpu.pressure)
                )
            }
            .frame(width: 278)
        }
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
                        .font(.system(size: 18, weight: .semibold, design: .rounded))
                }
                Spacer()
                ZStack {
                    Circle().fill(CleanerTheme.mint.opacity(0.14))
                    Image(systemName: "internaldrive.fill")
                        .font(.system(size: 22, weight: .semibold))
                        .foregroundStyle(CleanerTheme.mint)
                }
                .frame(width: 48, height: 48)
            }

            Spacer(minLength: 18)

            HStack(alignment: .firstTextBaseline, spacing: 7) {
                Text(Formatting.bytes(disk.availableBytes))
                    .font(.system(size: 38, weight: .bold, design: .rounded))
                    .tracking(-1.25)
                    .monospacedDigit()
                Text("free")
                    .font(.headline)
                    .foregroundStyle(.secondary)
            }

            Spacer(minLength: 16)

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
        .padding(20)
        .frame(minHeight: 206)
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
            .frame(width: 45, height: 45)

            VStack(alignment: .leading, spacing: 3) {
                Text(eyebrow)
                    .font(.system(size: 9, weight: .bold, design: .rounded))
                    .tracking(1.0)
                    .foregroundStyle(tint)
                Text(value)
                    .font(.system(size: 22, weight: .bold, design: .rounded))
                    .tracking(-0.35)
                    .monospacedDigit()
                Text(detail)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            Spacer(minLength: 0)
        }
        .padding(15)
        .frame(maxWidth: .infinity, minHeight: 97, alignment: .leading)
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

    private var maximumMemory: UInt64 { processes.map(\.memoryBytes).max() ?? 1 }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(alignment: .firstTextBaseline) {
                VStack(alignment: .leading, spacing: 3) {
                    Text("What’s active")
                        .font(.system(size: 18, weight: .semibold, design: .rounded))
                    Text("Largest memory and CPU consumers right now")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Text("MEMORY  ·  CPU  ·  SAFETY")
                    .font(.system(size: 9, weight: .bold, design: .rounded))
                    .tracking(1.05)
                    .foregroundStyle(.tertiary)
            }
            .padding(.horizontal, 18)
            .padding(.vertical, 15)

            Divider().opacity(0.55)

            LazyVStack(spacing: 1) {
                ForEach(processes) { process in
                    ProcessRow(process: process, maximumMemory: maximumMemory)
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
                Text("PID \(process.pid)")
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
            .frame(width: 105)

            Text(process.cpuPercent, format: .number.precision(.fractionLength(1)))
                .font(.caption.monospacedDigit())
                .foregroundStyle(.secondary)
                .frame(width: 46, alignment: .trailing)

            HStack(spacing: 6) {
                Image(systemName: process.protected ? "lock.fill" : "eye.fill")
                    .font(.system(size: 9, weight: .bold))
                Text(process.protected ? "Protected" : "Review")
                    .font(.caption2.weight(.semibold))
            }
            .foregroundStyle(process.protected ? CleanerTheme.mint : .secondary)
            .padding(.horizontal, 9)
            .padding(.vertical, 5)
            .background((process.protected ? CleanerTheme.mint : Color.secondary).opacity(0.09), in: Capsule())
            .frame(width: 92)
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
