import AppKit
import SwiftUI

private enum StorageFilter: String, CaseIterable, Identifiable {
    case all = "All"
    case growing = "Growing"
    case reviewable = "Reviewable"
    case protected = "Protected"
    var id: Self { self }
}

struct StorageView: View {
    @EnvironmentObject private var model: AppModel
    @State private var query = ""
    @State private var filter: StorageFilter = .all
    @State private var selectedID: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            header
            if let storage = model.storage {
                summary(storage)
                controls(storage)
                Table(visible(storage), selection: $selectedID) {
                    TableColumn("Location") { hotspot in
                        VStack(alignment: .leading, spacing: 2) {
                            Text(hotspot.displayName).fontWeight(.medium)
                            Text(hotspot.path).font(.caption2.monospaced()).foregroundStyle(.secondary).lineLimit(1)
                        }
                    }
                    TableColumn("Current") { Text(Formatting.bytes($0.bytes)).monospacedDigit() }
                        .width(100)
                    TableColumn("Change") { hotspot in
                        Text(hotspot.comparisonState == "newly-tracked" ? "New path" : Formatting.signedBytes(hotspot.deltaBytes))
                            .foregroundStyle(deltaColor(hotspot.deltaBytes))
                            .monospacedDigit()
                    }
                    .width(105)
                    TableColumn("Safety") { hotspot in
                        Label(dispositionLabel(hotspot.disposition), systemImage: dispositionSymbol(hotspot.disposition))
                            .foregroundStyle(dispositionColor(hotspot.disposition))
                    }
                    .width(115)
                }
                .frame(minHeight: 260)
                if let selected = storage.hotspots.first(where: { $0.id == selectedID }) {
                    detail(selected)
                }
                HStack {
                    Text("\(visible(storage).count) of \(storage.hotspots.count) measured paths")
                    Spacer()
                    Text("Allocated bytes · read-only evidence")
                }
                .font(.caption2)
                .foregroundStyle(.secondary)
                if !storage.warnings.isEmpty {
                    DisclosureGroup("\(storage.warnings.count) incomplete measurements") {
                        ForEach(storage.warnings, id: \.self) { Text($0).font(.caption).textSelection(.enabled) }
                    }
                }
            } else if model.isBusy {
                ProgressView("Measuring known developer storage…").frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                NativeEmptyState("No storage scan yet", systemImage: "internaldrive", description: "Run a read-only scan to record the first local checkpoint.") {
                    Button("Analyze Storage") { Task { await model.analyzeStorage() } }
                }
            }
        }
        .padding(22)
        .navigationTitle("Storage")
        .alert(item: $model.pendingToolPlan) { plan in
            Alert(
                title: Text("Confirm tool cleanup"),
                message: Text("\(plan.kind) will operate only on \(plan.targetPath). Maximum measured size: \(Formatting.bytes(plan.bytesBefore))."),
                primaryButton: .destructive(Text("Confirm")) { Task { await model.executePendingToolCleanup() } },
                secondaryButton: .cancel { model.pendingToolPlan = nil }
            )
        }
    }

    private var header: some View {
        HStack(alignment: .bottom) {
            VStack(alignment: .leading, spacing: 6) {
                Text("STORAGE FORENSICS")
                    .font(.system(size: 10, weight: .bold, design: .rounded))
                    .tracking(1.8)
                    .foregroundStyle(CleanerTheme.mint)
                Text("Where the space went")
                    .font(.system(size: 29, weight: .bold, design: .rounded))
                    .tracking(-0.6)
                Text("Measured developer paths, compared checkpoint by checkpoint.").foregroundStyle(.secondary)
            }
            Spacer()
            Button("Analyze Again", systemImage: "arrow.clockwise") { Task { await model.analyzeStorage() } }
                .disabled(model.isBusy)
                .buttonStyle(.borderedProminent)
                .tint(CleanerTheme.mint)
                .controlSize(.large)
        }
    }

    private func summary(_ storage: StorageAnalysis) -> some View {
        HStack(spacing: 10) {
            SummaryCell("Available", Formatting.bytes(storage.availableBytes), color: .primary)
            SummaryCell("Whole disk", Formatting.signedBytes(storage.comparison?.volumeUsedDeltaBytes), color: deltaColor(storage.comparison?.volumeUsedDeltaBytes))
            SummaryCell("Measured paths", Formatting.signedBytes(storage.comparison?.measuredHotspotDeltaBytes), color: deltaColor(storage.comparison?.measuredHotspotDeltaBytes))
            SummaryCell("Accounting gap", Formatting.signedBytes(storage.comparison?.accountingGapBytes), color: .secondary)
        }
    }

    private func controls(_ storage: StorageAnalysis) -> some View {
        HStack {
            Picker("Filter", selection: $filter) {
                ForEach(StorageFilter.allCases) { Text($0.rawValue).tag($0) }
            }
            .pickerStyle(.segmented)
            .frame(maxWidth: 360)
            Spacer()
            Button("Set Baseline", systemImage: "scope") { Task { await model.setStorageBaseline() } }
            Button("Copy Report", systemImage: "doc.on.doc") {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(Formatting.storageReport(storage), forType: .string)
                model.notice = "Storage report copied to the clipboard."
            }
        }
        .searchable(text: $query, placement: .toolbar, prompt: "Search path")
    }

    @ViewBuilder
    private func detail(_ hotspot: StorageHotspot) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 9) {
                HStack {
                    Text(hotspot.displayName).font(.headline)
                    Spacer()
                    Text(hotspot.category.replacingOccurrences(of: "-", with: " ")).font(.caption).foregroundStyle(.secondary)
                }
                Text(hotspot.path).font(.caption.monospaced()).textSelection(.enabled)
                Text(hotspot.reason).font(.callout)
                Divider()
                Text("Suggested next step").font(.caption.bold()).foregroundStyle(.secondary)
                Text(hotspot.recommendation).font(.callout)
                Text("Cost: \(hotspot.recoveryCost)").font(.caption).foregroundStyle(.secondary)
                HStack {
                    if hotspot.displayName == "uv package cache" {
                        Button("Prune Unused uv Cache") { Task { await model.prepareToolCleanup(kind: "uv") } }
                    }
                    if hotspot.displayName == "Simulator test recordings" {
                        Button("Review Old Recordings", role: .destructive) { Task { await model.prepareToolCleanup(kind: "simulator") } }
                    }
                    if hotspot.displayName == "Docker Desktop virtual disk" {
                        Button("Open Docker Desktop") { model.openApplication("Docker") }
                    }
                    if hotspot.displayName == "Apple simulator devices" {
                        Button("Open Xcode") { model.openApplication("Xcode") }
                    }
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private func visible(_ storage: StorageAnalysis) -> [StorageHotspot] {
        storage.hotspots.filter { hotspot in
            let matchesQuery = query.isEmpty || "\(hotspot.displayName) \(hotspot.path) \(hotspot.category)".localizedCaseInsensitiveContains(query)
            let matchesFilter = switch filter {
            case .all: true
            case .growing: (hotspot.deltaBytes ?? 0) > 0
            case .reviewable: hotspot.disposition == "reviewable"
            case .protected: hotspot.disposition == "protected"
            }
            return matchesQuery && matchesFilter
        }.sorted {
            if filter == .growing { return ($0.deltaBytes ?? .min) > ($1.deltaBytes ?? .min) }
            return $0.bytes > $1.bytes
        }
    }
}

private struct SummaryCell: View {
    let title: String
    let value: String
    let color: Color
    init(_ title: String, _ value: String, color: Color) { self.title = title; self.value = value; self.color = color }
    var body: some View {
        VStack(alignment: .leading, spacing: 7) {
            HStack(spacing: 6) {
                Circle().fill(color).frame(width: 6, height: 6)
                Text(title.uppercased())
                    .font(.system(size: 9, weight: .bold, design: .rounded))
                    .tracking(0.9)
                    .foregroundStyle(.secondary)
            }
            Text(value)
                .font(.system(size: 17, weight: .semibold, design: .rounded))
                .foregroundStyle(color)
                .monospacedDigit()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(13)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 15, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 15, style: .continuous)
                .stroke(color.opacity(0.16), lineWidth: 1)
        }
    }
}

private func deltaColor(_ value: Int64?) -> Color { guard let value else { return .secondary }; return value > 0 ? .orange : value < 0 ? .green : .secondary }
private func dispositionLabel(_ value: String) -> String { value == "tool-managed" ? "Owning tool" : value.capitalized }
private func dispositionSymbol(_ value: String) -> String { value == "protected" ? "lock.fill" : value == "tool-managed" ? "wrench.and.screwdriver" : "eye" }
private func dispositionColor(_ value: String) -> Color { value == "protected" ? .green : value == "tool-managed" ? .blue : .orange }
