import AppKit
import Combine
import Foundation
import UserNotifications

@MainActor
final class AppModel: ObservableObject {
    @Published var selection: SidebarItem? = .overview {
        didSet {
            if let selection {
                UserDefaults.standard.set(selection.rawValue, forKey: "lastSidebarItem")
            }
        }
    }
    @Published private(set) var snapshot: SystemSnapshot?
    @Published private(set) var storage: StorageAnalysis?
    @Published private(set) var scan: ScanReport?
    @Published private(set) var history: [ExecutionReceipt] = []
    @Published private(set) var toolHistory: [ToolCleanupReceipt] = []
    @Published private(set) var quarantine: [QuarantineEntry] = []
    @Published private(set) var isBusy = false
    @Published var notice = "Ready. Nothing is cleaned automatically."
    @Published var error: String?
    @Published var pendingCleanupPlan: CleanupPlan?
    @Published var pendingQuarantinePlan: QuarantineActionPlan?
    @Published var pendingToolPlan: ToolCleanupPlan?

    private let engine: EngineClient?
    private var monitorTask: Task<Void, Never>?
    private var lastPressure = "normal"

    init(engine: EngineClient? = nil) {
        if let savedSelection = UserDefaults.standard.string(forKey: "lastSidebarItem"),
           let sidebarItem = SidebarItem(rawValue: savedSelection) {
            selection = sidebarItem
        }
        if let engine {
            self.engine = engine
        } else {
            do {
                self.engine = try EngineClient()
            } catch {
                self.engine = nil
                self.error = error.localizedDescription
            }
        }
    }

    func startMonitoring() {
        guard monitorTask == nil else { return }
        monitorTask = Task { [weak self] in
            while !Task.isCancelled {
                await self?.refreshStatus(showBusy: false)
                try? await Task.sleep(for: .seconds(5))
            }
        }
        Task {
            await refreshCollections()
            let defaults = UserDefaults.standard
            let scanOnLaunch = defaults.object(forKey: "scanOnLaunch") == nil
                ? true
                : defaults.bool(forKey: "scanOnLaunch")
            if scanOnLaunch {
                await analyzeStorage()
                await scanCleanup()
            }
        }
    }

    func refreshStatus(showBusy: Bool = true) async {
        guard let engine else { return }
        if showBusy { isBusy = true }
        defer { if showBusy { isBusy = false } }
        do {
            let next: SystemSnapshot = try await engine.run(.status)
            snapshot = next
            await notifyIfNeeded(next.memory.pressure)
        } catch {
            self.error = error.localizedDescription
        }
    }

    func analyzeStorage() async {
        await perform("Measuring known storage paths…") { engine in
            let result: StorageAnalysis = try await engine.run(.storageAnalyze)
            storage = result
            notice = "Measured \(result.hotspots.count) known storage locations. Nothing was deleted."
        }
    }

    func setStorageBaseline() async {
        guard let id = storage?.snapshotId else { return }
        await perform("Saving storage baseline…") { engine in
            let _: BaselineAcknowledgement = try await engine.run(.storageSetBaseline(id))
            notice = "Baseline saved. Future scans will compare against this checkpoint."
        }
    }

    func scanCleanup() async {
        await perform("Scanning approved cleanup locations…") { engine in
            let result: ScanReport = try await engine.run(.scan)
            scan = result
            notice = "Found \(result.candidates.count) reviewable cleanup candidates."
        }
    }

    func prepareCleanup(candidateIDs: [String], dryRun: Bool = false) async {
        guard !candidateIDs.isEmpty else { return }
        await perform("Preparing an exact cleanup plan…") { engine in
            let plan: CleanupPlan = try await engine.run(.prepareClean(candidateIDs, dryRun: dryRun))
            pendingCleanupPlan = plan
        }
    }

    func executePendingCleanup() async {
        guard let plan = pendingCleanupPlan else { return }
        pendingCleanupPlan = nil
        await perform("Revalidating and executing the approved plan…") { engine in
            let receipt: ExecutionReceipt = try await engine.run(.executeClean(plan.id))
            notice = "Moved \(receipt.items.count) item(s) to recoverable quarantine."
            await refreshCollections(using: engine)
            let result: ScanReport = try await engine.run(.scan)
            scan = result
        }
    }

    func prepareQuarantine(_ entry: QuarantineEntry, action: String) async {
        await perform("Preparing \(action)…") { engine in
            let plan: QuarantineActionPlan = try await engine.run(.prepareQuarantine(entry.id, action: action))
            pendingQuarantinePlan = plan
        }
    }

    func executePendingQuarantine() async {
        guard let plan = pendingQuarantinePlan else { return }
        pendingQuarantinePlan = nil
        await perform("Revalidating quarantine action…") { engine in
            let _: QuarantineEntry = try await engine.run(.executeQuarantine(plan.id))
            notice = plan.action == "restore" ? "Item restored." : "Item permanently purged."
            await refreshCollections(using: engine)
        }
    }

    func prepareToolCleanup(kind: String) async {
        await perform("Preparing tool-owned cleanup…") { engine in
            let command: EngineCommand = kind == "uv" ? .prepareUVPrune : .prepareSimulatorRecordings
            let plan: ToolCleanupPlan = try await engine.run(command)
            pendingToolPlan = plan
        }
    }

    func executePendingToolCleanup() async {
        guard let plan = pendingToolPlan else { return }
        pendingToolPlan = nil
        await perform("Running verified tool cleanup…") { engine in
            let receipt: ToolCleanupReceipt = try await engine.run(.executeToolCleanup(plan.id))
            notice = receipt.summary
            await refreshCollections(using: engine)
            let result: StorageAnalysis = try await engine.run(.storageAnalyze)
            storage = result
        }
    }

    func refreshCollections() async {
        guard let engine else { return }
        await refreshCollections(using: engine)
    }

    func openApplication(_ name: String) {
        let path = switch name {
        case "Docker": "/Applications/Docker.app"
        case "Xcode": "/Applications/Xcode.app"
        default: "/System/Applications/Utilities/\(name).app"
        }
        let configuration = NSWorkspace.OpenConfiguration()
        NSWorkspace.shared.openApplication(at: URL(fileURLWithPath: path), configuration: configuration)
    }

    private func refreshCollections(using engine: EngineClient) async {
        do {
            async let nextHistory: [ExecutionReceipt] = engine.run(.history(20))
            async let nextTools: [ToolCleanupReceipt] = engine.run(.toolHistory(20))
            async let nextQuarantine: [QuarantineEntry] = engine.run(.quarantineList(100))
            let values = try await (nextHistory, nextTools, nextQuarantine)
            history = values.0
            toolHistory = values.1
            quarantine = values.2
        } catch {
            self.error = error.localizedDescription
        }
    }

    private func perform(
        _ progress: String,
        operation: (EngineClient) async throws -> Void
    ) async {
        guard let engine else { return }
        isBusy = true
        error = nil
        notice = progress
        defer { isBusy = false }
        do {
            try await operation(engine)
        } catch {
            self.error = error.localizedDescription
        }
    }

    private func notifyIfNeeded(_ pressure: String) async {
        defer { lastPressure = pressure }
        guard pressure != "normal", pressure != lastPressure else { return }
        let settings = await UNUserNotificationCenter.current().notificationSettings()
        guard settings.authorizationStatus == .authorized else { return }
        let content = UNMutableNotificationContent()
        content.title = pressure == "critical" ? "Memory pressure is critical" : "Memory pressure is elevated"
        content.body = "Review the largest unprotected apps. Mac Cleaner will not close developer work automatically."
        try? await UNUserNotificationCenter.current().add(UNNotificationRequest(identifier: "memory-\(pressure)", content: content, trigger: nil))
    }
}
