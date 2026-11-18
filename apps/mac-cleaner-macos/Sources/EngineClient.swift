import Foundation

enum EngineCommand: Sendable {
    case status
    case storageAnalyze
    case storageSetBaseline(UUID)
    case scan
    case prepareClean([String], dryRun: Bool)
    case executeClean(UUID)
    case history(Int)
    case quarantineList(Int)
    case prepareQuarantine(UUID, action: String)
    case executeQuarantine(UUID)
    case prepareUVPrune
    case prepareSimulatorRecordings
    case executeToolCleanup(UUID)
    case toolHistory(Int)
    case autoSettings
    case autoSettingsSet(String)
    case autoTick
    case autoReportQuit(String, UInt64)
    case autoLog(Int)
    case autoProtectedApps
    case energy

    var arguments: [String] {
        switch self {
        case .status: ["status"]
        case .storageAnalyze: ["storage-analyze"]
        case .storageSetBaseline(let id): ["storage-set-baseline", id.uuidString]
        case .scan: ["scan"]
        case .prepareClean(let ids, let dryRun):
            ["prepare-clean"] + ids.flatMap { ["--candidate", $0] } + (dryRun ? ["--dry-run"] : [])
        case .executeClean(let id): ["execute-clean", id.uuidString]
        case .history(let limit): ["history", "--limit", String(min(max(limit, 1), 100))]
        case .quarantineList(let limit): ["quarantine-list", "--limit", String(min(max(limit, 1), 100))]
        case .prepareQuarantine(let id, let action): ["prepare-quarantine", id.uuidString, action]
        case .executeQuarantine(let id): ["execute-quarantine", id.uuidString]
        case .prepareUVPrune: ["prepare-uv-prune"]
        case .prepareSimulatorRecordings: ["prepare-simulator-recordings"]
        case .executeToolCleanup(let id): ["execute-tool-cleanup", id.uuidString]
        case .toolHistory(let limit): ["tool-history", "--limit", String(min(max(limit, 1), 100))]
        case .autoSettings: ["auto-settings"]
        case .autoSettingsSet(let json): ["auto-settings-set", json]
        case .autoTick: ["auto-tick"]
        case .autoReportQuit(let app, let bytes): ["auto-report-quit", "--", app, String(bytes)]
        case .autoLog(let limit): ["auto-log", "--limit", String(min(max(limit, 1), 100))]
        case .autoProtectedApps: ["auto-protected-apps"]
        case .energy: ["energy"]
        }
    }
}

enum EngineClientError: LocalizedError {
    case helperMissing
    case helperOutsideBundle
    case invalidProtocol(Int)
    case failed(String)
    case invalidResponse

    var errorDescription: String? {
        switch self {
        case .helperMissing: "The bundled Rust engine is missing. Reinstall Mac Cleaner."
        case .helperOutsideBundle: "Mac Cleaner refused an engine outside its signed app bundle."
        case .invalidProtocol(let version): "Unsupported Rust engine protocol version \(version)."
        case .failed(let message): message
        case .invalidResponse: "The Rust engine returned an invalid response."
        }
    }
}

struct EngineClient: Sendable {
    private let executableURL: URL

    init(bundle: Bundle = .main) throws {
        let url = bundle.bundleURL
            .appendingPathComponent("Contents", isDirectory: true)
            .appendingPathComponent("Helpers", isDirectory: true)
            .appendingPathComponent("mac-cleaner-engine", isDirectory: false)
            .resolvingSymlinksInPath()
        guard FileManager.default.isExecutableFile(atPath: url.path) else {
            throw EngineClientError.helperMissing
        }
        let bundlePath = bundle.bundleURL.resolvingSymlinksInPath().path + "/"
        guard url.path.hasPrefix(bundlePath) else {
            throw EngineClientError.helperOutsideBundle
        }
        executableURL = url
    }

    init(executableURL: URL) {
        self.executableURL = executableURL
    }

    func run<Value: Decodable>(
        _ command: EngineCommand,
        as type: Value.Type = Value.self
    ) async throws -> Value {
        let result = try await Self.execute(executableURL: executableURL, arguments: command.arguments)
        guard result.status == 0 else {
            if let failure = try? Self.decoder.decode(EngineFailure.self, from: result.error) {
                throw EngineClientError.failed(failure.error)
            }
            throw EngineClientError.failed(String(decoding: result.error, as: UTF8.self))
        }
        let envelope = try Self.decoder.decode(EngineEnvelope<Value>.self, from: result.output)
        guard envelope.protocolVersion == 1 else {
            throw EngineClientError.invalidProtocol(envelope.protocolVersion)
        }
        guard envelope.ok else { throw EngineClientError.invalidResponse }
        return envelope.data
    }

    static func decode<Value: Decodable>(_ type: Value.Type, from data: Data) throws -> Value {
        try decoder.decode(EngineEnvelope<Value>.self, from: data).data
    }

    private static let decoder: JSONDecoder = {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return decoder
    }()

    private static func execute(executableURL: URL, arguments: [String]) async throws
        -> (status: Int32, output: Data, error: Data) {
        try await Task.detached(priority: .userInitiated) {
            let process = Process()
            let output = Pipe()
            let error = Pipe()
            process.executableURL = executableURL
            process.arguments = arguments
            process.standardOutput = output
            process.standardError = error
            process.standardInput = FileHandle.nullDevice
            try process.run()
            process.waitUntilExit()
            return (
                process.terminationStatus,
                output.fileHandleForReading.readDataToEndOfFile(),
                error.fileHandleForReading.readDataToEndOfFile()
            )
        }.value
    }
}
