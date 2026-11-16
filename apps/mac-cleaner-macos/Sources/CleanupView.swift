import SwiftUI

struct CleanupView: View {
    @EnvironmentObject private var model: AppModel
    @State private var selected = Set<String>()

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                header
                cleanupSummary
                recommendedActions
                approvedFiles
            }
            .padding(.horizontal, 26)
            .padding(.top, 22)
            .padding(.bottom, 30)
        }
        .navigationTitle("Cleanup")
        .alert(item: $model.pendingCleanupPlan) { plan in
            Alert(
                title: Text(plan.executionMode == "dry_run" ? "Validate this plan?" : "Move items to recovery?"),
                message: Text("\(plan.items.count) exact item(s), \(Formatting.bytes(plan.totalBytes)). Rust will reject stale or changed paths."),
                primaryButton: plan.executionMode == "dry_run"
                    ? .default(Text("Validate")) { Task { await model.executePendingCleanup() } }
                    : .destructive(Text("Move to Recovery")) { Task { await model.executePendingCleanup() } },
                secondaryButton: .cancel { model.pendingCleanupPlan = nil }
            )
        }
        .alert(item: $model.pendingToolPlan) { plan in
            Alert(
                title: Text("Confirm tool-owned cleanup"),
                message: Text("\(plan.kind) will operate only on \(plan.targetPath). Maximum measured size: \(Formatting.bytes(plan.bytesBefore))."),
                primaryButton: .destructive(Text("Confirm")) { Task { await model.executePendingToolCleanup() } },
                secondaryButton: .cancel { model.pendingToolPlan = nil }
            )
        }
    }

    private var header: some View {
        HStack(alignment: .bottom, spacing: 18) {
            VStack(alignment: .leading, spacing: 6) {
                Text("CLEANUP CENTER")
                    .font(.system(size: 10, weight: .bold, design: .rounded))
                    .tracking(1.8)
                    .foregroundStyle(CleanerTheme.coral)
                Text("Free space without breaking your tools")
                    .font(.system(size: 29, weight: .bold, design: .rounded))
                    .tracking(-0.6)
                Text("Review generated files, tool-owned caches, and old test artifacts. Every destructive action asks first.")
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button("Find opportunities", systemImage: "magnifyingglass") {
                Task {
                    await model.analyzeStorage()
                    await model.scanCleanup()
                }
            }
            .disabled(model.isBusy)
            .buttonStyle(.borderedProminent)
            .tint(CleanerTheme.coral)
            .controlSize(.large)
        }
    }

    private var cleanupSummary: some View {
        HStack(spacing: 10) {
            CleanupMetric(
                title: "Needs review",
                value: Formatting.bytes(model.storage?.reviewableBytes ?? 0),
                detail: "Evidence, not an automatic deletion total",
                symbol: "eye",
                tint: CleanerTheme.amber
            )
            CleanupMetric(
                title: "Approved files",
                value: "\(model.scan?.candidates.count ?? 0)",
                detail: model.scan == nil ? "Run the safe file scan" : "Old enough and inside the compiled catalog",
                symbol: "checkmark.shield",
                tint: CleanerTheme.mint
            )
            CleanupMetric(
                title: "Held for recovery",
                value: Formatting.bytes(quarantineBytes),
                detail: "Still uses disk until permanently purged",
                symbol: "archivebox",
                tint: CleanerTheme.violet
            )
        }
    }

    private var recommendedActions: some View {
        VStack(alignment: .leading, spacing: 11) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Recommended actions")
                        .font(.system(size: 18, weight: .semibold, design: .rounded))
                    Text("Owning tools do the cleanup where raw deletion would be risky.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Button("Review all measured paths") { model.selection = .storage }
                    .buttonStyle(.plain)
                    .foregroundStyle(CleanerTheme.azure)
            }

            HStack(spacing: 10) {
                CleanupActionCard(
                    symbol: "video.badge.checkmark",
                    title: "Old simulator videos",
                    detail: simulatorDetail,
                    actionTitle: "Review recordings",
                    tint: CleanerTheme.coral,
                    enabled: simulatorHotspot != nil && !model.isBusy
                ) {
                    Task { await model.prepareToolCleanup(kind: "simulator") }
                }
                CleanupActionCard(
                    symbol: "shippingbox",
                    title: "Unused uv downloads",
                    detail: uvDetail,
                    actionTitle: "Prune with uv",
                    tint: CleanerTheme.azure,
                    enabled: uvHotspot != nil && !model.isBusy
                ) {
                    Task { await model.prepareToolCleanup(kind: "uv") }
                }
                CleanupActionCard(
                    symbol: "hammer",
                    title: "Old Xcode builds",
                    detail: xcodeDetail,
                    actionTitle: "Scan old builds",
                    tint: CleanerTheme.amber,
                    enabled: !model.isBusy
                ) {
                    Task { await model.scanCleanup() }
                }
                CleanupActionCard(
                    symbol: "externaldrive.connected.to.line.below",
                    title: "Docker storage",
                    detail: dockerDetail,
                    actionTitle: "Open Docker",
                    tint: CleanerTheme.mint,
                    enabled: true
                ) {
                    model.openApplication("Docker")
                }
            }
        }
    }

    private var approvedFiles: some View {
        VStack(alignment: .leading, spacing: 11) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Approved file cleanup")
                        .font(.system(size: 18, weight: .semibold, design: .rounded))
                    Text("Only direct children of reviewed catalog locations can appear here.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Button("Scan again", systemImage: "arrow.clockwise") { Task { await model.scanCleanup() } }
                    .disabled(model.isBusy)
            }

            if let scan = model.scan {
                if scan.candidates.isEmpty {
                    HStack(spacing: 13) {
                        ZStack {
                            RoundedRectangle(cornerRadius: 12, style: .continuous)
                                .fill(CleanerTheme.mint.opacity(0.13))
                            Image(systemName: "checkmark.shield")
                                .foregroundStyle(CleanerTheme.mint)
                        }
                        .frame(width: 44, height: 44)
                        VStack(alignment: .leading, spacing: 3) {
                            Text("No old files qualify for direct cleanup")
                                .font(.headline)
                            Text("Use the tool-owned actions above or review Storage for generated project data.")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                        Spacer()
                        Button("Open Storage") { model.selection = .storage }
                    }
                    .padding(15)
                    .background(CleanerTheme.mint.opacity(0.06), in: RoundedRectangle(cornerRadius: 16, style: .continuous))
                } else {
                    VStack(spacing: 0) {
                        ForEach(scan.candidates) { candidate in
                            cleanupCandidate(candidate)
                            if candidate.id != scan.candidates.last?.id { Divider() }
                        }
                    }
                    .padding(.horizontal, 14)
                    .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 17, style: .continuous))

                    HStack {
                        Text("\(selected.count) selected · \(Formatting.bytes(selectedBytes(scan)))")
                            .font(.callout)
                            .foregroundStyle(.secondary)
                        Spacer()
                        Button("Validate only") { Task { await model.prepareCleanup(candidateIDs: Array(selected), dryRun: true) } }
                            .disabled(selected.isEmpty || model.isBusy)
                        Button("Review cleanup", role: .destructive) { Task { await model.prepareCleanup(candidateIDs: Array(selected)) } }
                            .disabled(selected.isEmpty || model.isBusy)
                    }
                }

                if !scan.skipped.isEmpty {
                    DisclosureGroup("\(scan.skipped.count) protected or skipped locations") {
                        ForEach(scan.skipped) { item in
                            VStack(alignment: .leading) {
                                Text(item.path).font(.caption.monospaced()).textSelection(.enabled)
                                Text(item.reason).font(.caption2).foregroundStyle(.secondary)
                            }
                            .padding(.vertical, 3)
                        }
                    }
                }
            } else {
                HStack(spacing: 13) {
                    Image(systemName: "checklist")
                        .font(.system(size: 24, weight: .semibold))
                        .foregroundStyle(CleanerTheme.coral)
                    VStack(alignment: .leading, spacing: 3) {
                        Text("Safe file scan has not run yet").font(.headline)
                        Text("Scan Xcode build output, CocoaPods downloads, and Mac Cleaner’s own cache using fixed Rust rules.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                    Button("Scan approved locations") { Task { await model.scanCleanup() } }
                        .buttonStyle(.borderedProminent)
                        .tint(CleanerTheme.coral)
                }
                .padding(16)
                .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 17, style: .continuous))
            }
        }
    }

    private func cleanupCandidate(_ candidate: CleanupCandidate) -> some View {
        Toggle(isOn: selectionBinding(candidate.id)) {
            HStack(alignment: .top, spacing: 12) {
                Image(systemName: "doc.badge.gearshape")
                    .foregroundStyle(CleanerTheme.amber)
                VStack(alignment: .leading, spacing: 4) {
                    HStack {
                        Text(candidate.displayName).fontWeight(.medium)
                        Spacer()
                        Text(Formatting.bytes(candidate.bytes)).fontWeight(.semibold).monospacedDigit()
                    }
                    Text(candidate.lexicalPath).font(.caption2.monospaced()).foregroundStyle(.secondary).lineLimit(1)
                    Text(candidate.reason).font(.caption).foregroundStyle(.secondary)
                }
            }
            .padding(.vertical, 8)
        }
        .toggleStyle(.checkbox)
    }

    private var quarantineBytes: UInt64 {
        model.quarantine.reduce(0) { $0 &+ $1.bytes }
    }

    private var simulatorHotspot: StorageHotspot? {
        model.storage?.hotspots.first { $0.displayName == "Simulator test recordings" }
    }

    private var uvHotspot: StorageHotspot? {
        model.storage?.hotspots.first { $0.displayName == "uv package cache" }
    }

    private var xcodeHotspot: StorageHotspot? {
        model.storage?.hotspots.first { $0.displayName == "Xcode DerivedData" }
    }

    private var dockerHotspot: StorageHotspot? {
        model.storage?.hotspots.first { $0.displayName == "Docker Desktop virtual disk" }
    }

    private var simulatorDetail: String {
        simulatorHotspot.map { "\(Formatting.bytes($0.bytes)) of verified recordings older than 72 hours" }
            ?? "Analyze storage to check for old test recordings"
    }

    private var uvDetail: String {
        uvHotspot.map { "\(Formatting.bytes($0.bytes)) measured; uv removes only unused entries" }
            ?? "Analyze storage to measure the uv cache"
    }

    private var xcodeDetail: String {
        xcodeHotspot.map { "\(Formatting.bytes($0.bytes)) measured; only projects idle for 14 days qualify" }
            ?? "Find old per-project DerivedData without removing Xcode"
    }

    private var dockerDetail: String {
        dockerHotspot.map { "\(Formatting.bytes($0.bytes)) shared by images, containers, and volumes" }
            ?? "Review images and volumes in their owning application"
    }

    private func selectionBinding(_ id: String) -> Binding<Bool> {
        Binding(
            get: { selected.contains(id) },
            set: { enabled in
                if enabled { selected.insert(id) } else { selected.remove(id) }
            }
        )
    }

    private func selectedBytes(_ scan: ScanReport) -> UInt64 {
        scan.candidates.filter { selected.contains($0.id) }.reduce(0) { $0 &+ $1.bytes }
    }
}

private struct CleanupMetric: View {
    let title: String
    let value: String
    let detail: String
    let symbol: String
    let tint: Color

    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                RoundedRectangle(cornerRadius: 11, style: .continuous).fill(tint.opacity(0.14))
                Image(systemName: symbol).foregroundStyle(tint)
            }
            .frame(width: 42, height: 42)
            VStack(alignment: .leading, spacing: 2) {
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
                    .lineLimit(1)
            }
            Spacer(minLength: 0)
        }
        .padding(14)
        .frame(maxWidth: .infinity, minHeight: 82, alignment: .leading)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 17, style: .continuous))
        .overlay { RoundedRectangle(cornerRadius: 17, style: .continuous).stroke(tint.opacity(0.16), lineWidth: 1) }
    }
}

private struct CleanupActionCard: View {
    let symbol: String
    let title: String
    let detail: String
    let actionTitle: String
    let tint: Color
    let enabled: Bool
    let action: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 9) {
            Image(systemName: symbol)
                .font(.system(size: 19, weight: .semibold))
                .foregroundStyle(tint)
            Text(title).font(.headline)
            Text(detail)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(3)
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 3)
            Button(actionTitle, action: action)
                .buttonStyle(.bordered)
                .disabled(!enabled)
        }
        .padding(15)
        .frame(maxWidth: .infinity, minHeight: 162, alignment: .topLeading)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 17, style: .continuous))
        .overlay { RoundedRectangle(cornerRadius: 17, style: .continuous).stroke(tint.opacity(0.14), lineWidth: 1) }
    }
}
