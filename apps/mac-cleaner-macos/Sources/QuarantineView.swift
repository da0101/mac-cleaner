import SwiftUI

struct QuarantineView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                header
                recoverySummary
                recoveryContent
            }
            .padding(.horizontal, 26)
            .padding(.top, 22)
            .padding(.bottom, 30)
        }
        .navigationTitle("Recovery")
        .alert(item: $model.pendingQuarantinePlan) { plan in
            Alert(
                title: Text(plan.action == "restore" ? "Restore this item?" : "Permanently erase this item?"),
                message: Text(plan.action == "restore" ? plan.entry.originalPath : "This frees \(Formatting.bytes(plan.entry.bytes)) but cannot be undone."),
                primaryButton: plan.action == "restore"
                    ? .default(Text("Restore")) { Task { await model.executePendingQuarantine() } }
                    : .destructive(Text("Erase Permanently")) { Task { await model.executePendingQuarantine() } },
                secondaryButton: .cancel { model.pendingQuarantinePlan = nil }
            )
        }
    }

    private var header: some View {
        HStack(alignment: .bottom) {
            VStack(alignment: .leading, spacing: 6) {
                Text("RECOVERY VAULT")
                    .font(.system(size: 10, weight: .bold, design: .rounded))
                    .tracking(1.8)
                    .foregroundStyle(CleanerTheme.violet)
                Text("Undo a cleanup—or finish freeing space")
                    .font(.system(size: 29, weight: .bold, design: .rounded))
                    .tracking(-0.6)
                Text("Moved files stay recoverable here. They consume disk space until you restore or permanently erase them.")
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button("Refresh", systemImage: "arrow.clockwise") { Task { await model.refreshCollections() } }
                .buttonStyle(.bordered)
                .controlSize(.large)
        }
    }

    private var recoverySummary: some View {
        HStack(spacing: 10) {
            RecoveryMetric(title: "Items held", value: "\(model.quarantine.count)", detail: "Recoverable cleanup entries", tint: CleanerTheme.violet)
            RecoveryMetric(title: "Space still occupied", value: Formatting.bytes(totalBytes), detail: "Erase items to reclaim this space", tint: totalBytes > 0 ? CleanerTheme.amber : CleanerTheme.mint)
            RecoveryMetric(title: "Safety", value: "Exact paths", detail: "Rust revalidates before restore or erase", tint: CleanerTheme.azure)
        }
    }

    @ViewBuilder
    private var recoveryContent: some View {
        if model.quarantine.isEmpty {
            HStack(spacing: 16) {
                ZStack {
                    RoundedRectangle(cornerRadius: 16, style: .continuous)
                        .fill(CleanerTheme.mint.opacity(0.13))
                    Image(systemName: "archivebox")
                        .font(.system(size: 24, weight: .semibold))
                        .foregroundStyle(CleanerTheme.mint)
                }
                .frame(width: 58, height: 58)
                VStack(alignment: .leading, spacing: 4) {
                    Text("Nothing is waiting for recovery")
                        .font(.system(size: 18, weight: .semibold, design: .rounded))
                    Text("This is healthy. Cleanup moves approved files here first so you can undo mistakes before permanently reclaiming space.")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Button("Find cleanup candidates") {
                    model.selection = .cleanup
                    Task { await model.scanCleanup() }
                }
                .buttonStyle(.borderedProminent)
                .tint(CleanerTheme.coral)
            }
            .padding(18)
            .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 20, style: .continuous))
        } else {
            VStack(spacing: 0) {
                ForEach(model.quarantine) { entry in
                    HStack(alignment: .top, spacing: 13) {
                        ZStack {
                            RoundedRectangle(cornerRadius: 10, style: .continuous)
                                .fill(CleanerTheme.violet.opacity(0.13))
                            Image(systemName: "archivebox.fill").foregroundStyle(CleanerTheme.violet)
                        }
                        .frame(width: 38, height: 38)
                        VStack(alignment: .leading, spacing: 4) {
                            Text(entry.originalPath).font(.callout.monospaced()).lineLimit(1)
                            Text("\(Formatting.bytes(entry.bytes)) · \(entry.files) file(s)")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                            Text("Held at \(entry.quarantinePath)")
                                .font(.caption2.monospaced())
                                .foregroundStyle(.tertiary)
                                .lineLimit(1)
                        }
                        Spacer()
                        Button("Restore") { Task { await model.prepareQuarantine(entry, action: "restore") } }
                        Button("Erase", role: .destructive) { Task { await model.prepareQuarantine(entry, action: "purge") } }
                    }
                    .padding(14)
                    if entry.id != model.quarantine.last?.id { Divider() }
                }
            }
            .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 18, style: .continuous))
        }
    }

    private var totalBytes: UInt64 {
        model.quarantine.reduce(0) { $0 &+ $1.bytes }
    }
}

private struct RecoveryMetric: View {
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
