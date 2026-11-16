import Foundation

enum Formatting {
    static func bytes(_ value: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useGB, .useMB, .useKB]
        formatter.countStyle = .file
        formatter.includesUnit = true
        formatter.isAdaptive = true
        return formatter.string(fromByteCount: Int64(clamping: value))
    }

    static func compactBytes(_ value: UInt64) -> String {
        let gigabytes = Double(value) / 1_000_000_000
        return gigabytes >= 100
            ? String(format: "%.0fG", gigabytes)
            : String(format: "%.1fG", gigabytes)
    }

    static func cpuPercent(_ value: Double, decimals: Int = 0) -> String {
        String(format: "%.*f%%", decimals, value)
    }

    static func signedBytes(_ value: Int64?) -> String {
        guard let value else { return "—" }
        if value == 0 { return "No change" }
        let magnitude = value == .min ? UInt64(Int64.max) + 1 : UInt64(abs(value))
        return "\(value > 0 ? "+" : "−")\(bytes(magnitude))"
    }

    static func percent(_ used: UInt64, of total: UInt64) -> Double {
        guard total > 0 else { return 0 }
        return min(max(Double(used) / Double(total), 0), 1)
    }

    static func storageReport(_ analysis: StorageAnalysis) -> String {
        var lines = [
            "Mac Cleaner storage forensics report",
            "Captured: \(analysis.capturedAt)",
            "Snapshot: \(analysis.snapshotId.uuidString)",
            "Available: \(bytes(analysis.availableBytes))",
        ]
        if let comparison = analysis.comparison {
            lines += [
                "Compared with \(comparison.kind): \(comparison.capturedAt)",
                "Whole-disk change: \(signedBytes(comparison.volumeUsedDeltaBytes))",
                "Measured paths: \(signedBytes(comparison.measuredHotspotDeltaBytes))",
                "Accounting gap: \(signedBytes(comparison.accountingGapBytes))",
            ]
        } else {
            lines.append("Compared with: first scan")
        }
        lines += ["", "Known paths:"]
        lines += analysis.hotspots.map { hotspot in
            let delta = hotspot.comparisonState == "newly-tracked" ? "newly tracked" : signedBytes(hotspot.deltaBytes)
            return "- \(hotspot.displayName) | \(bytes(hotspot.bytes)) | \(delta) | \(hotspot.disposition) | \(hotspot.path)"
        }
        if !analysis.warnings.isEmpty {
            lines += ["", "Incomplete measurements:"] + analysis.warnings.map { "- \($0)" }
        }
        return lines.joined(separator: "\n")
    }
}
