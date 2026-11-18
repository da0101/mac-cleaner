import SwiftUI

/// One checklist for everything Mac Cleaner can free, one action to do it.
struct CleanupView: View {
    @EnvironmentObject private var model: AppModel
    @State private var selected = Set<String>()
    @State private var confirmingClean = false
    @State private var confirmingEmptyRecovery = false
    @State private var showSkipped = false

    var body: some View {
        VStack(spacing: 0) {
            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    header
                    summary
                    content
                    ownedByOtherApps
                }
                .padding(.horizontal, 26)
                .padding(.top, 22)
                .padding(.bottom, 28)
            }
            if !candidates.isEmpty {
                actionBar
            }
        }
        .navigationTitle("Free Up Space")
        .onChange(of: model.scan?.completedAt) { _ in
            let valid = Set(candidates.map(\.id))
            selected = selected.intersection(valid)
        }
        .confirmationDialog(
            "Clean \(selected.count) item(s), \(Formatting.bytes(selectedBytes))?",
            isPresented: $confirmingClean,
            titleVisibility: .visible
        ) {
            Button("Move to Recovery (can undo)") {
                clean(erase: false)
            }
            Button("Delete permanently and free \(Formatting.bytes(selectedBytes)) now", role: .destructive) {
                clean(erase: true)
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text(confirmationMessage)
        }
        .confirmationDialog(
            "Permanently erase everything in Recovery?",
            isPresented: $confirmingEmptyRecovery,
            titleVisibility: .visible
        ) {
            Button("Erase \(Formatting.bytes(recoveryBytes))", role: .destructive) {
                Task { await model.emptyRecovery() }
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("\(model.quarantine.count) item(s) will be deleted and cannot be restored. Each is re-checked first.")
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

    // MARK: Header and summary

    private var header: some View {
        HStack(alignment: .bottom, spacing: 18) {
            VStack(alignment: .leading, spacing: 6) {
                Text("FREE UP SPACE")
                    .font(.system(size: 10, weight: .bold, design: .rounded))
                    .tracking(1.8)
                    .foregroundStyle(CleanerTheme.coral)
                Text("Pick what to remove")
                    .font(.system(size: 29, weight: .bold, design: .rounded))
                    .tracking(-0.6)
                Text("Everything here can be recreated by its tool. Sources, lockfiles and settings are never touched.")
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button(model.scan == nil ? "Scan" : "Rescan", systemImage: "arrow.clockwise") {
                Task {
                    await model.scanCleanup()
                    await model.analyzeStorage()
                }
            }
            .disabled(model.isBusy)
            .buttonStyle(.bordered)
            .controlSize(.large)
        }
    }

    private var summary: some View {
        HStack(spacing: 10) {
            SummaryTile(
                title: "Can be freed",
                value: model.scan == nil ? "—" : Formatting.bytes(totalBytes),
                detail: model.scan == nil ? "Scan to find out" : "\(candidates.count) item(s) found",
                tint: CleanerTheme.mint
            )
            SummaryTile(
                title: "Selected",
                value: Formatting.bytes(selectedBytes),
                detail: "\(selected.count) item(s)",
                tint: CleanerTheme.coral
            )
            SummaryTile(
                title: "In Recovery",
                value: Formatting.bytes(recoveryBytes),
                detail: recoveryBytes > 0 ? "Still using disk space" : "Nothing held",
                tint: CleanerTheme.violet
            ) {
                if recoveryBytes > 0 {
                    Button("Empty") { confirmingEmptyRecovery = true }
                        .disabled(model.isBusy)
                }
            }
        }
    }

    // MARK: Checklist

    @ViewBuilder
    private var content: some View {
        if model.scan == nil {
            EmptyPanel(
                symbol: model.isBusy ? "hourglass" : "magnifyingglass",
                title: model.isBusy ? "Scanning…" : "Not scanned yet",
                detail: model.isBusy
                    ? "Measuring caches and project folders. Large node_modules folders take a little while."
                    : "Scan to see what can be freed. Scanning never removes anything."
            ) {
                if !model.isBusy {
                    Button("Scan now") { Task { await model.scanCleanup() } }
                        .buttonStyle(.borderedProminent)
                        .tint(CleanerTheme.coral)
                }
            }
        } else if candidates.isEmpty {
            EmptyPanel(
                symbol: "checkmark.seal",
                title: "Nothing to free right now",
                detail: "No caches or generated project folders were found. See below for space owned by other apps."
            ) { EmptyView() }
        } else {
            ForEach(groups) { group in
                groupSection(group)
            }
        }

        if let skipped = model.scan?.skipped, !skipped.isEmpty {
            DisclosureGroup(isExpanded: $showSkipped) {
                VStack(alignment: .leading, spacing: 6) {
                    ForEach(skipped) { item in
                        VStack(alignment: .leading, spacing: 1) {
                            Text(item.path)
                                .font(.caption.monospaced())
                                .lineLimit(1)
                                .truncationMode(.middle)
                                .textSelection(.enabled)
                            Text(item.reason)
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
                .padding(.top, 6)
            } label: {
                Text("\(skipped.count) item(s) not offered: \(skipSummary(skipped))")
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private func groupSection(_ group: CandidateGroup) -> some View {
        VStack(alignment: .leading, spacing: 9) {
            HStack(alignment: .firstTextBaseline) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(group.title)
                        .font(.system(size: 17, weight: .semibold, design: .rounded))
                    Text(group.subtitle)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Text(Formatting.bytes(group.bytes))
                    .font(.callout.weight(.semibold).monospacedDigit())
                    .foregroundStyle(.secondary)
                Button(group.allSelected(selected) ? "Deselect all" : "Select all") {
                    if group.allSelected(selected) {
                        selected.subtract(group.selectableIDs)
                    } else {
                        selected.formUnion(group.selectableIDs)
                    }
                }
                .buttonStyle(.plain)
                .foregroundStyle(CleanerTheme.azure)
                .disabled(group.selectableIDs.isEmpty)
            }

            VStack(spacing: 1) {
                ForEach(group.items) { candidate in
                    candidateRow(candidate)
                }
            }
            .padding(5)
            .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 15, style: .continuous))
            .overlay {
                RoundedRectangle(cornerRadius: 15, style: .continuous)
                    .stroke(Color.primary.opacity(0.08), lineWidth: 1)
            }
        }
    }

    private func candidateRow(_ candidate: CleanupCandidate) -> some View {
        let usage = CandidateUsage(candidate)
        let isOn = selected.contains(candidate.id)
        return HStack(spacing: 12) {
            Toggle("", isOn: Binding(
                get: { isOn },
                set: { value in
                    if value { selected.insert(candidate.id) } else { selected.remove(candidate.id) }
                }
            ))
            .toggleStyle(.checkbox)
            .labelsHidden()

            VStack(alignment: .leading, spacing: 3) {
                HStack(spacing: 7) {
                    Text(title(for: candidate))
                        .font(.system(size: 13, weight: .semibold, design: .rounded))
                    Text(kind(for: candidate))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    if let badge = usage.badge {
                        Text(badge.text)
                            .font(.system(size: 8, weight: .bold, design: .rounded))
                            .tracking(0.6)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .foregroundStyle(badge.tint)
                            .background(badge.tint.opacity(0.12), in: Capsule())
                    }
                }
                Text("\(usage.lastUsed) · \(candidate.recovery)")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .contentShape(Rectangle())
            .onTapGesture {
                if isOn { selected.remove(candidate.id) } else { selected.insert(candidate.id) }
            }

            Text(Formatting.bytes(candidate.bytes))
                .font(.callout.weight(.semibold).monospacedDigit())
                .frame(width: 84, alignment: .trailing)

            Button {
                model.revealInFinder(candidate.lexicalPath)
            } label: {
                Image(systemName: "magnifyingglass.circle")
            }
            .buttonStyle(.plain)
            .foregroundStyle(.secondary)
            .help("Show in Finder: \(candidate.lexicalPath)")
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 7)
        .background(
            isOn ? CleanerTheme.coral.opacity(0.07) : Color.clear,
            in: RoundedRectangle(cornerRadius: 10, style: .continuous)
        )
        .help(candidate.reason)
    }

    private var actionBar: some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 1) {
                Text(selected.isEmpty ? "Select items to clean" : "\(selected.count) selected · \(Formatting.bytes(selectedBytes))")
                    .font(.callout.weight(.semibold))
                Text(selectedInUseCount > 0
                     ? "\(selectedInUseCount) selected item(s) were used in the last day. Close those projects first."
                     : "You'll choose between Recovery (undoable) and deleting permanently.")
                    .font(.caption)
                    .foregroundStyle(selectedInUseCount > 0 ? CleanerTheme.amber : .secondary)
            }
            Spacer()
            if model.isBusy {
                ProgressView().controlSize(.small)
            }
            Button("Clean Selected…") { confirmingClean = true }
                .buttonStyle(.borderedProminent)
                .tint(CleanerTheme.coral)
                .controlSize(.large)
                .disabled(selected.isEmpty || model.isBusy)
                .keyboardShortcut(.defaultAction)
        }
        .padding(.horizontal, 26)
        .padding(.vertical, 12)
        .background(.bar)
        .overlay(alignment: .top) { Divider() }
    }

    // MARK: Space owned by other apps

    @ViewBuilder
    private var ownedByOtherApps: some View {
        let items = toolManaged
        if !items.isEmpty {
            VStack(alignment: .leading, spacing: 9) {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Managed by other apps")
                        .font(.system(size: 17, weight: .semibold, design: .rounded))
                    Text("Mac Cleaner won't delete these directly. Clean them from the app that owns them.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                VStack(spacing: 1) {
                    ForEach(items) { hotspot in
                        HStack(spacing: 12) {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(hotspot.displayName)
                                    .font(.system(size: 13, weight: .semibold, design: .rounded))
                                Text(hotspot.recommendation)
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                                    .lineLimit(2)
                            }
                            .frame(maxWidth: .infinity, alignment: .leading)
                            Text(Formatting.bytes(hotspot.bytes))
                                .font(.callout.weight(.semibold).monospacedDigit())
                                .frame(width: 84, alignment: .trailing)
                            ownerAction(for: hotspot)
                                .frame(width: 128, alignment: .trailing)
                        }
                        .padding(.horizontal, 10)
                        .padding(.vertical, 8)
                    }
                }
                .padding(5)
                .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 15, style: .continuous))
            }
        }
    }

    @ViewBuilder
    private func ownerAction(for hotspot: StorageHotspot) -> some View {
        switch hotspot.displayName {
        case "Docker Desktop virtual disk":
            Button("Open Docker") { model.openApplication("Docker") }
        case "Xcode device support", "Apple simulator devices":
            Button("Open Xcode") { model.openApplication("Xcode") }
        case "uv package cache":
            Button("Prune with uv") { Task { await model.prepareToolCleanup(kind: "uv") } }
                .disabled(model.isBusy)
        case "Simulator test recordings":
            Button("Review recordings") { Task { await model.prepareToolCleanup(kind: "simulator") } }
                .disabled(model.isBusy)
        default:
            Button("Show in Finder") { model.revealInFinder(hotspot.path) }
        }
    }

    // MARK: Derived data

    private var candidates: [CleanupCandidate] { model.scan?.candidates ?? [] }

    private var totalBytes: UInt64 { candidates.reduce(0) { $0 &+ $1.bytes } }

    private var selectedBytes: UInt64 {
        candidates.filter { selected.contains($0.id) }.reduce(0) { $0 &+ $1.bytes }
    }

    private var selectedInUseCount: Int {
        candidates.filter { selected.contains($0.id) && CandidateUsage($0).inUse }.count
    }

    private var recoveryBytes: UInt64 { model.quarantine.reduce(0) { $0 &+ $1.bytes } }

    private var toolManaged: [StorageHotspot] {
        // A location that Rust now offers as a removable item is not repeated
        // as a read-only row.
        let offered = Set(candidates.map(\.targetId))
        let promoted: [String: String] = [
            "Xcode device support": "xcode-device-support",
            "Chrome on-device model": "chrome-on-device-model",
        ]
        return (model.storage?.hotspots ?? [])
            .filter { hotspot in
                if let target = promoted[hotspot.displayName], offered.contains(target) { return false }
                return (hotspot.disposition == "tool-managed" && hotspot.bytes >= 100_000_000)
                    || (hotspot.displayName == "Simulator test recordings")
                    || (hotspot.displayName == "uv package cache" && hotspot.bytes >= 50_000_000)
            }
            .sorted { $0.bytes > $1.bytes }
    }

    private static let xcodeTargets: Set<String> = ["xcode-derived-data", "xcode-device-support", "xctest-clone-devices"]

    private var groups: [CandidateGroup] {
        let definitions: [(String, String, String, (String) -> Bool)] = [
            ("caches", "Download caches",
             "Shared package and browser downloads. Cheapest to remove: tools fetch them again when needed.",
             { !$0.hasPrefix("project-") && !Self.xcodeTargets.contains($0) }),
            ("projects", "Project dependencies and builds",
             "Per-project folders. The next install or build recreates them, which can take minutes.",
             { $0.hasPrefix("project-") }),
            ("xcode", "Old Xcode data",
             "Unused build output and device symbols. Xcode rebuilds or copies them again when needed.",
             { Self.xcodeTargets.contains($0) }),
        ]
        return definitions.compactMap { id, title, subtitle, matches in
            // Ties (several Zero KB rows) fall back to the path so rows never swap places on a rescan.
            let items = candidates.filter { matches($0.targetId) }.sorted {
                $0.bytes != $1.bytes ? $0.bytes > $1.bytes : $0.lexicalPath < $1.lexicalPath
            }
            return items.isEmpty ? nil : CandidateGroup(id: id, title: title, subtitle: subtitle, items: items)
        }
    }

    private var confirmationMessage: String {
        let chosen = candidates.filter { selected.contains($0.id) }.sorted { $0.bytes > $1.bytes }
        var lines = chosen.prefix(8).map { "• \(title(for: $0)) \(kind(for: $0)): \(Formatting.bytes($0.bytes))" }
        if chosen.count > 8 { lines.append("• and \(chosen.count - 8) more") }
        lines.append("")
        lines.append("Recovery keeps the files (and their disk space) so you can restore them. Deleting permanently frees the space now.")
        return lines.joined(separator: "\n")
    }

    private func clean(erase: Bool) {
        let ids = Array(selected)
        Task {
            await model.cleanSelected(candidateIDs: ids, eraseImmediately: erase)
            selected.removeAll()
        }
    }

    private func title(for candidate: CleanupCandidate) -> String {
        if candidate.targetId.hasPrefix("project-") {
            // Nested entries such as `ios/build` or `.yarn/berry-cache` span two components.
            let nested: Set<String> = ["project-xcode-derived-data", "project-ios-build", "project-yarn-offline-cache"]
            let parts = candidate.relativeEntry.split(separator: "/")
            return parts.dropLast(nested.contains(candidate.targetId) ? 2 : 1).joined(separator: "/")
        }
        if candidate.targetId == "xcode-device-support" {
            return candidate.relativeEntry
        }
        if candidate.targetId == "xcode-derived-data" {
            return candidate.relativeEntry.split(separator: "-").dropLast().joined(separator: "-")
        }
        return candidate.displayName.components(separatedBy: " — ").first ?? candidate.displayName
    }

    private func kind(for candidate: CleanupCandidate) -> String {
        switch candidate.targetId {
        case "project-node-modules": "node_modules"
        case "project-rust-target": "Rust target"
        case "project-dart-tool": ".dart_tool"
        case "project-flutter-build": "Flutter build"
        case "xcode-derived-data": "DerivedData"
        case "project-xcode-derived-data": "Xcode DerivedData"
        case "project-ios-build": "iOS build"
        case "project-yarn-offline-cache": "Yarn cache"
        case "xcode-device-support": "device symbols"
        default: ""
        }
    }

    private func skipSummary(_ skipped: [ScanSkip]) -> String {
        let recent = skipped.filter { $0.reason.hasPrefix("used within") }.count
        let other = skipped.count - recent
        var parts: [String] = []
        if recent > 0 { parts.append("\(recent) used recently") }
        if other > 0 { parts.append("\(other) failed a safety check") }
        return parts.joined(separator: ", ")
    }
}

// MARK: - Supporting types

private struct CandidateGroup: Identifiable {
    let id: String
    let title: String
    let subtitle: String
    let items: [CleanupCandidate]

    var bytes: UInt64 { items.reduce(0) { $0 &+ $1.bytes } }
    var selectableIDs: Set<String> { Set(items.map(\.id)) }
    func allSelected(_ selection: Set<String>) -> Bool {
        !items.isEmpty && selectableIDs.isSubset(of: selection)
    }
}

private struct CandidateUsage {
    let lastUsed: String
    let inUse: Bool
    let badge: (text: String, tint: Color)?

    init(_ candidate: CleanupCandidate) {
        let date = Self.parse(candidate.lastModified)
        let age = date.map { Date().timeIntervalSince($0) } ?? .infinity
        if let date {
            let relative = RelativeDateTimeFormatter()
            relative.unitsStyle = .full
            lastUsed = "Changed \(relative.localizedString(for: date, relativeTo: Date()))"
        } else {
            lastUsed = "Last change unknown"
        }
        inUse = age < 86_400
        if candidate.lexicalPath.hasSuffix("/mac-cleaner-full-rust-rewrite/target") {
            badge = ("THIS APP'S BUILD", CleanerTheme.amber)
        } else if inUse {
            badge = ("IN USE TODAY", CleanerTheme.amber)
        } else if age > 30 * 86_400 {
            badge = ("UNUSED 30+ DAYS", CleanerTheme.mint)
        } else {
            badge = nil
        }
    }

    private static func parse(_ value: String) -> Date? {
        let precise = ISO8601DateFormatter()
        precise.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        if let date = precise.date(from: value) { return date }
        return ISO8601DateFormatter().date(from: value)
    }
}

private struct SummaryTile<Accessory: View>: View {
    let title: String
    let value: String
    let detail: String
    let tint: Color
    @ViewBuilder var accessory: Accessory

    init(title: String, value: String, detail: String, tint: Color, @ViewBuilder accessory: () -> Accessory = { EmptyView() }) {
        self.title = title
        self.value = value
        self.detail = detail
        self.tint = tint
        self.accessory = accessory()
    }

    var body: some View {
        HStack(alignment: .center) {
            VStack(alignment: .leading, spacing: 2) {
                Text(title.uppercased())
                    .font(.system(size: 9, weight: .bold, design: .rounded))
                    .tracking(0.9)
                    .foregroundStyle(tint)
                Text(value)
                    .font(.system(size: 22, weight: .bold, design: .rounded))
                    .monospacedDigit()
                Text(detail)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            Spacer(minLength: 0)
            accessory
        }
        .padding(14)
        .frame(maxWidth: .infinity, minHeight: 78, alignment: .leading)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 16, style: .continuous))
        .overlay { RoundedRectangle(cornerRadius: 16, style: .continuous).stroke(tint.opacity(0.16), lineWidth: 1) }
    }
}

private struct EmptyPanel<Action: View>: View {
    let symbol: String
    let title: String
    let detail: String
    @ViewBuilder var action: Action

    var body: some View {
        HStack(spacing: 14) {
            Image(systemName: symbol)
                .font(.system(size: 22, weight: .semibold))
                .foregroundStyle(CleanerTheme.coral)
                .frame(width: 40)
            VStack(alignment: .leading, spacing: 3) {
                Text(title).font(.headline)
                Text(detail).font(.caption).foregroundStyle(.secondary)
            }
            Spacer()
            action
        }
        .padding(16)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 16, style: .continuous))
    }
}
