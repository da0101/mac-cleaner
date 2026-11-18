import SwiftUI

/// Read-only view of what is using CPU and memory, with every app's helper
/// processes added together. The only action is a normal quit for apps Rust
/// marked quittable; developer tools are shown but never offered for quitting.
struct EnergyView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                header
                if let report = model.energy {
                    summary(report)
                    appList(report)
                } else {
                    HStack(spacing: 10) {
                        ProgressView().controlSize(.small)
                        Text("Measuring what is using your CPU…").foregroundStyle(.secondary)
                    }
                    .padding(16)
                }
            }
            .padding(.horizontal, 26)
            .padding(.top, 22)
            .padding(.bottom, 28)
        }
        .navigationTitle("Energy")
        .task {
            // Sampling takes about a second, so it only runs while this screen is open.
            while !Task.isCancelled {
                await model.refreshEnergy()
                try? await Task.sleep(for: .seconds(5))
            }
        }
    }

    // MARK: Header

    private var header: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("ENERGY")
                .font(.system(size: 11, weight: .bold, design: .rounded))
                .tracking(1.6)
                .foregroundStyle(CleanerTheme.coral)
            Text("What is using your CPU and battery")
                .font(.system(size: 30, weight: .bold, design: .rounded))
            Text("Each app is shown with all its helper processes added together. Developer tools can be heavy, but they are never quit from here.")
                .foregroundStyle(.secondary)
        }
    }

    // MARK: Summary

    private func summary(_ report: EnergyReport) -> some View {
        let busiest = report.apps.first
        return HStack(spacing: 14) {
            statCard(
                title: "TOTAL CPU",
                value: Formatting.cpuPercent(report.totalCpuPercent),
                detail: "\(report.logicalCores) cores in total",
                tint: CleanerTheme.azure
            )
            if let busiest {
                statCard(
                    title: "HEAVIEST APP",
                    value: busiest.name,
                    detail: "\(Formatting.cpuPercent(busiest.cpuPercent)) CPU · \(Formatting.bytes(busiest.memoryBytes))",
                    tint: CleanerTheme.coral
                )
            }
        }
    }

    private func statCard(title: String, value: String, detail: String, tint: Color) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.system(size: 10, weight: .bold, design: .rounded))
                .tracking(1.2)
                .foregroundStyle(tint)
            Text(value)
                .font(.system(size: 22, weight: .bold, design: .rounded))
                .lineLimit(1)
            Text(detail).font(.caption).foregroundStyle(.secondary)
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 17, style: .continuous))
        .overlay { RoundedRectangle(cornerRadius: 17, style: .continuous).stroke(tint.opacity(0.18), lineWidth: 1) }
    }

    // MARK: App list

    private func appList(_ report: EnergyReport) -> some View {
        let peak = max(report.apps.map(\.cpuPercent).max() ?? 1, 1)
        return VStack(alignment: .leading, spacing: 0) {
            ForEach(Array(report.apps.enumerated()), id: \.element.id) { index, app in
                if index > 0 { Divider().opacity(0.4) }
                row(app, peak: peak)
            }
        }
        .padding(.vertical, 6)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 17, style: .continuous))
        .overlay { RoundedRectangle(cornerRadius: 17, style: .continuous).stroke(Color.white.opacity(0.06), lineWidth: 1) }
    }

    private func row(_ app: AppEnergy, peak: Double) -> some View {
        HStack(spacing: 14) {
            VStack(alignment: .leading, spacing: 3) {
                HStack(spacing: 8) {
                    Text(app.name).font(.callout.weight(.semibold)).lineLimit(1)
                    if app.protected {
                        Text("PROTECTED")
                            .font(.system(size: 9, weight: .bold, design: .rounded))
                            .tracking(0.8)
                            .padding(.horizontal, 6).padding(.vertical, 2)
                            .background(CleanerTheme.violet.opacity(0.18), in: Capsule())
                            .foregroundStyle(CleanerTheme.violet)
                            .help(app.protectionReason ?? "Developer tool or system app. It is never quit automatically.")
                    }
                }
                Text(app.processCount > 1 ? "\(app.processCount) processes" : "1 process")
                    .font(.caption).foregroundStyle(.secondary)
            }
            .frame(minWidth: 180, alignment: .leading)

            GeometryReader { proxy in
                ZStack(alignment: .leading) {
                    Capsule().fill(Color.white.opacity(0.07))
                    Capsule()
                        .fill(tint(for: app.cpuPercent))
                        .frame(width: max(4, proxy.size.width * min(app.cpuPercent / peak, 1)))
                }
            }
            .frame(height: 6)

            Text(Formatting.cpuPercent(app.cpuPercent))
                .font(.callout.monospacedDigit())
                .frame(width: 64, alignment: .trailing)
            Text(Formatting.bytes(app.memoryBytes))
                .font(.callout.monospacedDigit())
                .foregroundStyle(.secondary)
                .frame(width: 84, alignment: .trailing)

            if app.canQuit {
                Button("Quit") { Task { await model.quitApp(app) } }
                    .controlSize(.small)
                    .help("Asks \(app.name) to quit normally, like ⌘Q. Unsaved documents still ask to save.")
            } else {
                Color.clear.frame(width: 44, height: 1)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
    }

    private func tint(for cpu: Double) -> Color {
        switch cpu {
        case 80...: CleanerTheme.coral
        case 30...: CleanerTheme.amber
        default: CleanerTheme.mint
        }
    }
}
