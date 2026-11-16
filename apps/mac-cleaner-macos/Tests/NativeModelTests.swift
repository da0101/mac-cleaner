import Foundation
import Testing
@testable import MacCleanerNative

@Test func decodesVersionedEngineEnvelope() throws {
    let json = Data(#"{"protocol_version":1,"ok":true,"data":{"snapshot_id":"5C8C58D2-945B-4B1A-AE42-E311081B767E"}}"#.utf8)
    let acknowledgement = try EngineClient.decode(BaselineAcknowledgement.self, from: json)
    #expect(acknowledgement.snapshotId.uuidString == "5C8C58D2-945B-4B1A-AE42-E311081B767E")
}

@Test func engineCommandsNeverAcceptFilesystemPaths() {
    let id = UUID(uuidString: "5C8C58D2-945B-4B1A-AE42-E311081B767E")!
    #expect(EngineCommand.executeClean(id).arguments == ["execute-clean", id.uuidString])
    #expect(!EngineCommand.executeClean(id).arguments.contains("/"))
}

@Test func storageReportIncludesEvidenceAndAccountingGap() throws {
    let json = Data(#"{"snapshot_id":"5C8C58D2-945B-4B1A-AE42-E311081B767E","catalog_version":2,"captured_at":"2026-11-16T12:00:00Z","total_bytes":1000,"available_bytes":300,"pressure":"healthy","reviewable_bytes":80,"hotspots":[{"id":"yarn","display_name":"Yarn","path":"/Users/test/.yarn","bytes":80,"category":"package-cache","disposition":"reviewable","comparison_state":"comparable","previous_bytes":40,"delta_bytes":40,"reason":"cache","recommendation":"use Yarn","recovery_cost":"download"}],"warnings":[],"comparison":{"snapshot_id":"59A8E638-7B02-49FE-A3A2-2769CE69B9DF","captured_at":"2026-11-15T12:00:00Z","kind":"previous","volume_used_delta_bytes":100,"measured_hotspot_delta_bytes":40,"accounting_gap_bytes":60,"comparable_hotspots":1,"newly_tracked_hotspots":0}}"#.utf8)
    let decoder = JSONDecoder()
    decoder.keyDecodingStrategy = .convertFromSnakeCase
    let analysis = try decoder.decode(StorageAnalysis.self, from: json)
    let report = Formatting.storageReport(analysis)
    #expect(report.contains("Accounting gap"))
    #expect(report.contains("/Users/test/.yarn"))
}
