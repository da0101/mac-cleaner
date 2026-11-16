import AppKit
import ServiceManagement
import SwiftUI
import UserNotifications

@MainActor
final class LoginItemController: ObservableObject {
    @Published private(set) var status = SMAppService.mainApp.status
    var enabled: Bool { status == .enabled }
    var statusText: String {
        switch status {
        case .enabled: "Enabled"
        case .requiresApproval: "Requires approval in System Settings"
        case .notFound: "Unavailable until installed as an application"
        default: "Disabled"
        }
    }
    func setEnabled(_ enabled: Bool) throws {
        if enabled { try SMAppService.mainApp.register() } else { try SMAppService.mainApp.unregister() }
        status = SMAppService.mainApp.status
    }
    func refresh() { status = SMAppService.mainApp.status }
}

struct SettingsView: View {
    @EnvironmentObject private var model: AppModel
    @AppStorage("appearanceMode") private var appearance = AppAppearance.system
    @AppStorage("scanOnLaunch") private var scanOnLaunch = true
    @StateObject private var login = LoginItemController()
    @State private var notificationStatus = "Not requested"

    var body: some View {
        Form {
            Section("Appearance") {
                Picker("Color mode", selection: $appearance) {
                    ForEach(AppAppearance.allCases) { mode in
                        Text(mode.title).tag(mode)
                    }
                }
                .pickerStyle(.segmented)
                Text(appearance == .system ? "Follows the current macOS appearance." : "Uses \(appearance.title.lowercased()) appearance until you change it.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Section("General") {
                Toggle("Analyze storage and cleanup opportunities on launch", isOn: $scanOnLaunch)
                Text("Mac Cleaner reloads local history, measures known storage paths, and refreshes the safe cleanup catalog. It never cleans automatically.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Toggle("Launch Mac Cleaner at login", isOn: Binding(
                    get: { login.enabled },
                    set: { value in
                        do { try login.setEnabled(value) } catch { model.error = error.localizedDescription }
                    }
                ))
                Text(login.statusText).font(.caption).foregroundStyle(.secondary)
                if login.status == .requiresApproval {
                    Button("Open Login Items Settings") { SMAppService.openSystemSettingsLoginItems() }
                }
            }
            Section("Folder access") {
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("macOS controls protected-folder permission")
                        Text("Permission persists for a stable signed app. Replacing an unsigned development build may require approval again.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                    Button("Open Full Disk Access") { openFullDiskAccess() }
                }
            }
            Section("Notifications") {
                HStack {
                    VStack(alignment: .leading) {
                        Text("Memory-pressure alerts")
                        Text(notificationStatus).font(.caption).foregroundStyle(.secondary)
                    }
                    Spacer()
                    Button("Allow Notifications") {
                        Task {
                            do {
                                let allowed = try await UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound])
                                notificationStatus = allowed ? "Allowed" : "Not allowed"
                            } catch { model.error = error.localizedDescription }
                        }
                    }
                }
            }
            Section("Storage history") {
                Text("Mac Cleaner retains at most 30 local storage snapshots. Reports contain paths and sizes, never file contents.")
                    .font(.caption).foregroundStyle(.secondary)
            }
            Section("Claude & Codex access") {
                HStack(spacing: 12) {
                    Image(systemName: MCPAccess.isAvailable ? "point.3.connected.trianglepath.dotted" : "exclamationmark.triangle")
                        .foregroundStyle(MCPAccess.isAvailable ? CleanerTheme.mint : CleanerTheme.amber)
                    VStack(alignment: .leading, spacing: 2) {
                        Text(MCPAccess.isAvailable ? "Protected MCP helper included" : "MCP helper is missing")
                        Text("Agents can inspect CPU and storage, then prepare exact cleanup plans. Destructive steps require your explicit plan confirmation.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                    Button("Copy Setup Commands") { copyMCPSetup() }
                        .disabled(!MCPAccess.isAvailable)
                }
            }
        }
        .formStyle(.grouped)
        .padding()
        .onAppear { login.refresh() }
    }

    private func copyMCPSetup() {
        MCPAccess.copy(MCPAccess.setupCommands)
        model.notice = "Claude and Codex MCP setup commands copied."
    }

    private func openFullDiskAccess() {
        guard let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles") else { return }
        NSWorkspace.shared.open(url)
    }
}

enum MCPAccess {
    static var helperPath: String {
        Bundle.main.bundleURL
            .appendingPathComponent("Contents/Helpers/mac-cleaner-mcp")
            .path
    }

    static var isAvailable: Bool {
        FileManager.default.isExecutableFile(atPath: helperPath)
    }

    static var setupCommands: String {
        let escapedPath = helperPath.replacingOccurrences(of: "'", with: "'\\''")
        return """
        codex mcp add mac-cleaner-storage -- '\(escapedPath)'
        claude mcp add --scope user --transport stdio mac-cleaner-storage -- '\(escapedPath)'
        """
    }

    static let inspectionPrompt = """
    Use mac-cleaner-storage. Call system_health and latest_storage_report first. Explain current CPU/RAM pressure and top consumers, rank storage paths and recent growth, and separate reviewable/tool-managed/protected data. If cleanup is appropriate, use only Mac Cleaner's prepare tools, show me the exact plan and recovery behavior, wait for my explicit approval, and only then call its matching execute tool with the returned confirmation. Never use raw deletion or arbitrary paths.
    """

    static func copy(_ text: String) {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(text, forType: .string)
    }
}

struct AgentAccessView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                header
                statusCard
                toolGrid
                workflowCard
            }
            .padding(.horizontal, 26)
            .padding(.top, 22)
            .padding(.bottom, 30)
        }
        .navigationTitle("Agents")
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("AGENT ACCESS")
                .font(.system(size: 10, weight: .bold, design: .rounded))
                .tracking(1.8)
                .foregroundStyle(CleanerTheme.azure)
            Text("Claude & Codex, connected to the evidence")
                .font(.system(size: 29, weight: .bold, design: .rounded))
                .tracking(-0.6)
            Text("Agents can inspect CPU and storage, then clean through the same exact-plan safety engine as this app.")
                .foregroundStyle(.secondary)
        }
    }

    private var statusCard: some View {
        HStack(spacing: 16) {
            ZStack {
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .fill((MCPAccess.isAvailable ? CleanerTheme.mint : CleanerTheme.amber).opacity(0.14))
                Image(systemName: MCPAccess.isAvailable ? "point.3.connected.trianglepath.dotted" : "exclamationmark.triangle")
                    .font(.system(size: 24, weight: .semibold))
                    .foregroundStyle(MCPAccess.isAvailable ? CleanerTheme.mint : CleanerTheme.amber)
            }
            .frame(width: 58, height: 58)

            VStack(alignment: .leading, spacing: 4) {
                Text(MCPAccess.isAvailable ? "Storage MCP is installed" : "Storage MCP helper is missing")
                    .font(.system(size: 18, weight: .semibold, design: .rounded))
                Text("Local stdio · exact plans · explicit confirmation · no arbitrary paths")
                    .font(.callout)
                    .foregroundStyle(.secondary)
                Text(MCPAccess.helperPath)
                    .font(.caption2.monospaced())
                    .foregroundStyle(.tertiary)
                    .lineLimit(1)
                    .textSelection(.enabled)
            }
            Spacer()
            Button("Copy setup") {
                MCPAccess.copy(MCPAccess.setupCommands)
                model.notice = "Claude and Codex MCP setup commands copied."
            }
            .disabled(!MCPAccess.isAvailable)
            .buttonStyle(.bordered)
            Button("Copy review prompt") {
                MCPAccess.copy(MCPAccess.inspectionPrompt)
                model.notice = "Agent storage-review prompt copied."
            }
            .buttonStyle(.borderedProminent)
            .tint(CleanerTheme.azure)
        }
        .padding(18)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 20, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 20, style: .continuous)
                .stroke(CleanerTheme.azure.opacity(0.18), lineWidth: 1)
        }
    }

    private var toolGrid: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("What agents can do")
                .font(.system(size: 18, weight: .semibold, design: .rounded))
            HStack(spacing: 10) {
                AgentToolCard(symbol: "internaldrive", title: "Storage evidence", detail: "Measured paths, growth, safety labels, and catalog candidates")
                AgentToolCard(symbol: "cpu", title: "CPU & memory", detail: "Live load, swap, pressure, and protected developer jobs")
                AgentToolCard(symbol: "checklist.checked", title: "Prepare cleanup", detail: "Five-minute plans from opaque Mac Cleaner IDs—never paths")
                AgentToolCard(symbol: "archivebox", title: "Recovery control", detail: "Restore or separately confirm permanent space reclamation")
            }
        }
    }

    private var workflowCard: some View {
        HStack(spacing: 14) {
            Image(systemName: "lock.shield")
                .font(.system(size: 24, weight: .semibold))
                .foregroundStyle(CleanerTheme.coral)
            VStack(alignment: .leading, spacing: 3) {
                Text("Agents can clean. You remain the authority.")
                    .font(.headline)
                Text("Claude or Codex must prepare and show an exact Rust-owned plan first. Execution is a separate destructive tool call with a plan-specific confirmation; protected and unknown data never enter the plan.")
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button("Analyze storage") {
                model.selection = .storage
                Task { await model.analyzeStorage() }
            }
            .buttonStyle(.bordered)
        }
        .padding(18)
        .background(CleanerTheme.coral.opacity(0.07), in: RoundedRectangle(cornerRadius: 18, style: .continuous))
    }
}

private struct AgentToolCard: View {
    let symbol: String
    let title: String
    let detail: String

    var body: some View {
        VStack(alignment: .leading, spacing: 9) {
            Image(systemName: symbol)
                .font(.system(size: 18, weight: .semibold))
                .foregroundStyle(CleanerTheme.azure)
            Text(title)
                .font(.headline)
            Text(detail)
                .font(.caption)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 0)
        }
        .padding(15)
        .frame(maxWidth: .infinity, minHeight: 126, alignment: .topLeading)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 16, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .stroke(Color.primary.opacity(0.07), lineWidth: 1)
        }
    }
}
