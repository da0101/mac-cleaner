import AppKit
import Combine
import SwiftUI

enum AppAppearance: String, CaseIterable, Identifiable {
    case system
    case light
    case dark

    var id: Self { self }
    var title: String { rawValue.capitalized }
    var colorScheme: ColorScheme? {
        switch self {
        case .system: nil
        case .light: .light
        case .dark: .dark
        }
    }
}

@MainActor
final class MacCleanerAppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem?
    private let statusPopover = NSPopover()
    private var snapshotCancellable: AnyCancellable?

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
        bringMainWindowForward(after: 0.2, attemptsRemaining: 12)
    }

    /// Set by the SwiftUI scene so AppKit can recreate a closed main window.
    var openMainWindow: (() -> Void)?

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        showMainWindow()
        return true
    }

    /// Brings the main window forward, recreating it if the user closed it.
    /// Closing a SwiftUI WindowGroup window destroys it, so looking for an
    /// existing window alone cannot reopen the app from the Dock or `open -a`.
    func showMainWindow() {
        if mainWindow != nil {
            bringMainWindowForward(attemptsRemaining: 4)
        } else {
            NSApp.activate(ignoringOtherApps: true)
            openMainWindow?()
            bringMainWindowForward(after: 0.15, attemptsRemaining: 8)
        }
    }

    private var mainWindow: NSWindow? {
        NSApp.windows.first { window in
            window.isVisible
                && window.canBecomeKey
                && !(window is NSPanel)
                && !String(describing: type(of: window)).contains("Popover")
        }
    }

    func configureMenuBar(model: AppModel) {
        guard statusItem == nil else { return }

        let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        item.autosaveName = "com.da0101.mac-cleaner.system-status"
        item.button?.target = self
        item.button?.action = #selector(toggleStatusPopover(_:))
        item.button?.sendAction(on: [.leftMouseUp])
        statusItem = item

        statusPopover.behavior = .transient
        statusPopover.animates = true
        statusPopover.contentSize = NSSize(width: 350, height: 470)
        let hosting = NSHostingController(rootView: MenuBarView().environmentObject(model))
        // The panel's height follows its content, so adding or hiding a section never clips it.
        hosting.sizingOptions = [.preferredContentSize]
        statusPopover.contentViewController = hosting

        snapshotCancellable = model.$snapshot
            .receive(on: RunLoop.main)
            .sink { [weak self] snapshot in self?.updateStatusItem(snapshot) }
    }

    @objc private func toggleStatusPopover(_ sender: NSStatusBarButton) {
        if statusPopover.isShown {
            statusPopover.performClose(sender)
        } else {
            statusPopover.show(relativeTo: sender.bounds, of: sender, preferredEdge: .minY)
        }
    }

    private func updateStatusItem(_ snapshot: SystemSnapshot?) {
        guard let item = statusItem, let button = item.button else { return }
        guard let snapshot else {
            button.title = "Mac Cleaner"
            item.length = NSStatusItem.variableLength
            return
        }

        let title = NSMutableAttributedString(string: "")
        title.append(statusIcon("memorychip", color: pressureColor(snapshot.memory.pressure)))
        title.append(statusText(" RAM \(Formatting.compactBytes(snapshot.memory.availableBytes))   "))
        title.append(statusIcon("cpu", color: pressureColor(snapshot.cpu.pressure)))
        title.append(statusText(" CPU \(Formatting.cpuPercent(snapshot.cpu.usagePercent))   "))
        title.append(statusIcon(
            "internaldrive.fill",
            color: storageColor(snapshot.disk.availableBytes, total: snapshot.disk.totalBytes)
        ))
        title.append(statusText(" SSD \(Formatting.compactBytes(snapshot.disk.availableBytes))"))

        button.attributedTitle = title
        button.toolTip = "RAM, CPU, and free SSD storage"
        button.setAccessibilityLabel(
            "Mac Cleaner, \(Formatting.bytes(snapshot.memory.availableBytes)) memory available, CPU \(Int(snapshot.cpu.usagePercent.rounded())) percent, \(Formatting.bytes(snapshot.disk.availableBytes)) storage free"
        )
        item.length = ceil(title.size().width) + 12
    }

    private func statusIcon(_ name: String, color: NSColor) -> NSAttributedString {
        guard let base = NSImage(systemSymbolName: name, accessibilityDescription: nil) else {
            return NSAttributedString(string: "")
        }
        let point = NSImage.SymbolConfiguration(pointSize: 11, weight: .semibold)
        let palette = NSImage.SymbolConfiguration(paletteColors: [color])
        let image = base.withSymbolConfiguration(point.applying(palette)) ?? base
        image.isTemplate = false
        image.size = NSSize(width: 13, height: 13)

        let attachment = NSTextAttachment()
        attachment.image = image
        attachment.bounds = NSRect(x: 0, y: -2, width: 13, height: 13)
        return NSAttributedString(attachment: attachment)
    }

    private func statusText(_ value: String) -> NSAttributedString {
        NSAttributedString(
            string: value,
            attributes: [
                .font: NSFont.monospacedDigitSystemFont(ofSize: 11, weight: .semibold),
                .foregroundColor: NSColor.labelColor
            ]
        )
    }

    private func pressureColor(_ pressure: String) -> NSColor {
        switch pressure {
        case "critical": .systemRed
        case "warning": .systemOrange
        default: .systemGreen
        }
    }

    private func storageColor(_ available: UInt64, total: UInt64) -> NSColor {
        let fraction = Formatting.percent(available, of: total)
        if fraction < 0.05 { return .systemRed }
        if fraction < 0.15 { return .systemOrange }
        return .systemGreen
    }

    private func bringMainWindowForward(after delay: TimeInterval = 0, attemptsRemaining: Int = 1) {
        DispatchQueue.main.asyncAfter(deadline: .now() + delay) {
            NSRunningApplication.current.activate(options: [.activateAllWindows, .activateIgnoringOtherApps])
            if let window = self.mainWindow {
                window.hasShadow = false
                window.collectionBehavior.insert(.moveToActiveSpace)
                window.makeKeyAndOrderFront(nil)
                window.orderFrontRegardless()
            } else if attemptsRemaining > 1 {
                self.bringMainWindowForward(after: 0.2, attemptsRemaining: attemptsRemaining - 1)
            }
        }
    }
}

@main
struct MacCleanerApp: App {
    @NSApplicationDelegateAdaptor(MacCleanerAppDelegate.self) private var appDelegate
    @StateObject private var model = AppModel()
    @AppStorage("appearanceMode") private var appearance = AppAppearance.system

    var body: some Scene {
        WindowGroup("Mac Cleaner", id: "main") {
            RootView()
                .environmentObject(model)
                .preferredColorScheme(appearance.colorScheme)
                .frame(minWidth: 900, minHeight: 640)
                .onAppear { appDelegate.configureMenuBar(model: model) }
                .background(WindowOpenerBridge(delegate: appDelegate))
        }
        .defaultSize(width: 1120, height: 760)
        .windowResizability(.contentMinSize)
        .commands {
            CommandGroup(after: .appInfo) {
                Button("Analyze Storage") { Task { await model.analyzeStorage() } }
                    .keyboardShortcut("r", modifiers: [.command, .shift])
            }
        }

        Settings {
            SettingsView()
                .environmentObject(model)
                .preferredColorScheme(appearance.colorScheme)
                .frame(width: 560, height: 510)
        }
    }

}

/// Hands SwiftUI's `openWindow` action to the AppKit delegate.
private struct WindowOpenerBridge: View {
    let delegate: MacCleanerAppDelegate
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        Color.clear
            .frame(width: 0, height: 0)
            .onAppear {
                delegate.openMainWindow = { openWindow(id: "main") }
            }
    }
}
