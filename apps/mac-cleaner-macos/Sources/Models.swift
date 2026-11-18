import Foundation

struct EngineEnvelope<Value: Decodable>: Decodable {
    let protocolVersion: Int
    let ok: Bool
    let data: Value
}

struct EngineFailure: Decodable {
    let protocolVersion: Int?
    let ok: Bool?
    let error: String
}

struct SystemSnapshot: Decodable {
    let capturedAt: String
    let cpu: CPUSnapshot
    let memory: MemorySnapshot
    let disk: DiskSnapshot
    let processes: [ProcessSnapshot]
}

struct CPUSnapshot: Decodable {
    let usagePercent: Double
    let logicalCores: UInt32
    let pressure: String
}

struct MemorySnapshot: Decodable {
    let totalBytes: UInt64
    let freeBytes: UInt64
    let availableBytes: UInt64
    let usedBytes: UInt64
    let swapUsedBytes: UInt64
    let reclaimableBytes: UInt64
    let compressedBytes: UInt64
    let fileBackedBytes: UInt64
    let pressure: String
}

struct DiskSnapshot: Decodable {
    let totalBytes: UInt64
    let availableBytes: UInt64
}

struct ProcessSnapshot: Decodable, Identifiable {
    let pid: UInt32
    let startTimeUnix: UInt64
    let name: String
    let executable: String?
    let memoryBytes: UInt64
    let cpuPercent: Double
    let protected: Bool
    let protectionReason: String?
    var id: String { "\(pid):\(startTimeUnix)" }
}

struct StorageAnalysis: Decodable {
    let snapshotId: UUID
    let catalogVersion: UInt32
    let capturedAt: String
    let totalBytes: UInt64
    let availableBytes: UInt64
    let pressure: String
    let reviewableBytes: UInt64
    let hotspots: [StorageHotspot]
    let warnings: [String]
    let comparison: StorageComparison?
}

struct StorageHotspot: Decodable, Identifiable {
    let id: String
    let displayName: String
    let path: String
    let bytes: UInt64
    let category: String
    let disposition: String
    let comparisonState: String
    let previousBytes: UInt64?
    let deltaBytes: Int64?
    let reason: String
    let recommendation: String
    let recoveryCost: String
}

struct StorageComparison: Decodable {
    let snapshotId: UUID
    let capturedAt: String
    let kind: String
    let volumeUsedDeltaBytes: Int64
    let measuredHotspotDeltaBytes: Int64
    let accountingGapBytes: Int64
    let comparableHotspots: UInt32
    let newlyTrackedHotspots: UInt32
}

struct ScanReport: Decodable {
    let startedAt: String
    let completedAt: String
    let candidates: [CleanupCandidate]
    let skipped: [ScanSkip]
}

struct CleanupCandidate: Decodable, Identifiable {
    let id: String
    let targetId: String
    let displayName: String
    let lexicalPath: String
    let resolvedPath: String
    let relativeEntry: String
    let category: String
    let reason: String
    let owner: String
    let recovery: String
    let safetyTier: String
    let bytes: UInt64
    let files: UInt64
    let lastModified: String
}

struct ScanSkip: Decodable, Identifiable {
    let targetId: String
    let path: String
    let reason: String
    var id: String { "\(targetId):\(path)" }
}

struct CleanupPlan: Decodable, Identifiable {
    let id: UUID
    let expiresAt: String
    let executionMode: String
    let items: [CleanupCandidate]
    let totalBytes: UInt64
    let totalFiles: UInt64
}

struct ExecutionReceipt: Decodable, Identifiable {
    let planId: UUID
    let mode: String
    let completedAt: String
    let items: [ReceiptItem]
    var id: UUID { planId }
}

struct ReceiptItem: Decodable {
    let candidateId: String
    let originalPath: String
    let quarantinePath: String?
    let bytes: UInt64
    let files: UInt64
}

struct QuarantineEntry: Decodable, Identifiable {
    let id: UUID
    let planId: UUID
    let candidateId: String
    let originalPath: String
    let quarantinePath: String
    let bytes: UInt64
    let files: UInt64
    let quarantinedAt: String
    let state: String
}

struct QuarantineActionPlan: Decodable, Identifiable {
    let id: UUID
    let action: String
    let expiresAt: String
    let entry: QuarantineEntry
}

struct ToolCleanupPlan: Decodable, Identifiable {
    let id: UUID
    let kind: String
    let targetPath: String
    let bytesBefore: UInt64
    let expiresAt: String
}

struct ToolCleanupReceipt: Decodable, Identifiable {
    let planId: UUID
    let kind: String
    let targetPath: String
    let bytesBefore: UInt64
    let bytesAfter: UInt64
    let reclaimedBytes: UInt64
    let completedAt: String
    let summary: String
    var id: UUID { planId }
}

struct BaselineAcknowledgement: Decodable {
    let snapshotId: UUID
}

enum SidebarItem: String, CaseIterable, Identifiable {
    case overview = "Overview"
    case cleanup = "Free Up Space"
    case storage = "Storage"
    case quarantine = "Recovery"
    case activity = "History"
    case energy = "Energy"
    case autoClean = "Auto-Clean"
    case agents = "Agents"
    var id: Self { self }
    var symbol: String {
        switch self {
        case .overview: "gauge.with.dots.needle.50percent"
        case .storage: "internaldrive"
        case .cleanup: "trash"
        case .quarantine: "archivebox"
        case .activity: "list.bullet.rectangle.portrait"
        case .energy: "battery.50percent"
        case .autoClean: "bolt.circle"
        case .agents: "point.3.connected.trianglepath.dotted"
        }
    }
}

// MARK: - Auto-clean

struct AutoRamSettings: Codable, Equatable, Sendable {
    var enabled = false
    var minAvailableBytes: UInt64 = 2 * 1_073_741_824
    var sustainedSecs: UInt64 = 120
    var cooldownSecs: UInt64 = 180
    var quitApps: [String] = []
}

struct AutoStorageSettings: Codable, Equatable, Sendable {
    var enabled = false
    var intervalHours: UInt64 = 6
    var lowFreeBytes: UInt64 = 40 * 1_073_741_824
    var minIdleHours: UInt64 = 24
    var targetIds: [String] = []
}

struct AutoCleanSettings: Codable, Equatable, Sendable {
    var ram = AutoRamSettings()
    var storage = AutoStorageSettings()
}

struct AutoQuitTarget: Decodable, Sendable {
    let appName: String
    let appPath: String
    let bytes: UInt64
    let pids: [UInt32]
}

/// Rust returns a tagged value; only the fields of the matching `kind` are set.
struct AutoRamDecision: Decodable, Sendable {
    let kind: String
    let secsRemaining: UInt64?
    let reason: String?
    let target: AutoQuitTarget?
}

struct AutoStorageReport: Decodable, Sendable {
    let items: UInt64
    let bytesFreed: UInt64
    let skipped: UInt64
}

struct AutoTickReport: Decodable, Sendable {
    let ram: AutoRamDecision
    let storage: AutoStorageReport?
}

struct AutoLogEntry: Decodable, Identifiable, Sendable {
    let at: String
    let kind: String
    let summary: String
    let bytes: UInt64
    var id: String { at + kind + summary }
}

struct AutoRecorded: Decodable, Sendable {
    let recorded: Bool
}

// MARK: - Energy

struct AppEnergy: Decodable, Identifiable, Sendable {
    let name: String
    let appPath: String?
    let processCount: UInt32
    let memoryBytes: UInt64
    let cpuPercent: Double
    let protected: Bool
    let protectionReason: String?
    let canQuit: Bool
    var id: String { appPath ?? "proc:\(name)" }
}

struct EnergyReport: Decodable, Sendable {
    let logicalCores: UInt32
    let totalCpuPercent: Double
    let apps: [AppEnergy]
}
