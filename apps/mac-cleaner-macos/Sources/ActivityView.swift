import SwiftUI

struct ActivityView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                header
                historySummary
                nextActions
                receiptList
            }
            .padding(.horizontal, 26)
            .padding(.top, 22)
            .padding(.bottom, 30)
        }
        .navigationTitle("History")
    }

    private var header: some View {
        HStack(alignment: .bottom) {
            VStack(alignment: .leading, spacing: 6) {
                Text("CLEANUP HISTORY")
                    .font(.system(size: 10, weight: .bold, design: .rounded))
                    .tracking(1.8)
                    .foregroundStyle(CleanerTheme.azure)
                Text("What changed, and how much came back")
                    .font(.system(size: 29, weight: .bold, design: .rounded))
                    .tracking(-0.6)
                Text("Every completed Rust operation is recorded locally so cleanup remains explainable.")
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button("Refresh", systemImage: "arrow.clockwise") { Task { await model.refreshCollections() } }
                .buttonStyle(.bordered)
                .controlSize(.large)
        }
    }

    private var historySummary: some View {
        HStack(spacing: 10) {
            HistoryMetric(title: "Tool space reclaimed", value: Formatting.bytes(toolReclaimed), detail: "Verified before-and-after measurements", tint: CleanerTheme.mint)
            HistoryMetric(title: "Maintenance runs", value: "\(model.toolHistory.count)", detail: "Owning-tool operations recorded", tint: CleanerTheme.azure)
            HistoryMetric(title: "Files moved safely", value: "\(movedItems)", detail: "Recoverable cleanup items", tint: CleanerTheme.violet)
        }
    }

    private var nextActions: some View {
        HStack(spacing: 14) {
            Image(systemName: "arrow.triangle.2.circlepath")
                .font(.system(size: 23, weight: .semibold))
                .foregroundStyle(CleanerTheme.coral)
            VStack(alignment: .leading, spacing: 3) {
                Text("Measure again after cleanup")
                    .font(.headline)
                Text("A new storage checkpoint confirms whether space was reclaimed or immediately regrew.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button("Find more cleanup") {
                model.selection = .cleanup
                Task {
                    await model.analyzeStorage()
                    await model.scanCleanup()
                }
            }
            .buttonStyle(.bordered)
            Button("Analyze storage") {
                model.selection = .storage
                Task { await model.analyzeStorage() }
            }
            .buttonStyle(.borderedProminent)
            .tint(CleanerTheme.azure)
        }
        .padding(16)
        .background(CleanerTheme.azure.opacity(0.07), in: RoundedRectangle(cornerRadius: 18, style: .continuous))
    }

    private var receiptList: some View {
        VStack(alignment: .leading, spacing: 11) {
            Text("Receipts")
                .font(.system(size: 18, weight: .semibold, design: .rounded))

            if model.history.isEmpty && model.toolHistory.isEmpty {
                HStack(spacing: 13) {
                    Image(systemName: "list.bullet.rectangle.portrait")
                        .font(.system(size: 23, weight: .semibold))
                        .foregroundStyle(CleanerTheme.azure)
                    VStack(alignment: .leading, spacing: 3) {
                        Text("No cleanup receipts yet").font(.headline)
                        Text("Completed cleanup and tool maintenance will appear here with paths, timestamps, and reclaimed bytes.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                    Button("Open Cleanup") { model.selection = .cleanup }
                }
                .padding(16)
                .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 17, style: .continuous))
            } else {
                VStack(spacing: 0) {
                    ForEach(model.history) { receipt in
                        ActivityRow(
                            symbol: receipt.mode == "dry_run" ? "checkmark.shield" : "archivebox",
                            title: receipt.mode == "dry_run" ? "Plan validated" : "Moved to recovery",
                            detail: "\(receipt.items.count) exact item(s)",
                            amount: Formatting.bytes(receipt.items.reduce(0) { $0 &+ $1.bytes }),
                            timestamp: receipt.completedAt,
                            tint: CleanerTheme.violet
                        )
                        Divider()
                    }
                    ForEach(Array(model.toolHistory.enumerated()), id: \.element.id) { index, receipt in
                        ActivityRow(
                            symbol: "wrench.and.screwdriver",
                            title: receipt.summary,
                            detail: receipt.targetPath,
                            amount: Formatting.bytes(receipt.reclaimedBytes),
                            timestamp: receipt.completedAt,
                            tint: CleanerTheme.azure
                        )
                        if index != model.toolHistory.count - 1 { Divider() }
                    }
                }
                .padding(.horizontal, 14)
                .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 17, style: .continuous))
            }
        }
    }

    private var toolReclaimed: UInt64 {
        model.toolHistory.reduce(0) { $0 &+ $1.reclaimedBytes }
    }

    private var movedItems: Int {
        model.history.filter { $0.mode != "dry_run" }.reduce(0) { $0 + $1.items.count }
    }
}

private struct HistoryMetric: View {
    let title: String
    let value: String
    let detail: String
    let tint: Color

    var body: some View {
        VStack(alignment: .leading, spacing: 5) {
            Text(title.uppercased())
                .font(.system(size: 9, weight: .bold, design: .rounded))
                .tracking(0.9)
                .foregroundStyle(tint)
            Text(value)
                .font(.system(size: 20, weight: .bold, design: .rounded))
                .monospacedDigit()
            Text(detail)
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
        .padding(14)
        .frame(maxWidth: .infinity, minHeight: 82, alignment: .leading)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 17, style: .continuous))
        .overlay { RoundedRectangle(cornerRadius: 17, style: .continuous).stroke(tint.opacity(0.16), lineWidth: 1) }
    }
}

private struct ActivityRow: View {
    let symbol: String
    let title: String
    let detail: String
    let amount: String
    let timestamp: String
    let tint: Color

    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                RoundedRectangle(cornerRadius: 9, style: .continuous)
                    .fill(tint.opacity(0.13))
                Image(systemName: symbol).foregroundStyle(tint)
            }
            .frame(width: 36, height: 36)
            VStack(alignment: .leading, spacing: 3) {
                Text(title).fontWeight(.medium)
                Text(detail).font(.caption).foregroundStyle(.secondary).lineLimit(1)
            }
            Spacer()
            VStack(alignment: .trailing, spacing: 3) {
                Text(amount).monospacedDigit()
                Text(timestamp).font(.caption2).foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 9)
    }
}
