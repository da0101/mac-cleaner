import AppKit
import SwiftUI

enum CleanerTheme {
    static let coral = Color(red: 0.98, green: 0.39, blue: 0.31)
    static let mint = Color(red: 0.31, green: 0.84, blue: 0.62)
    static let azure = Color(red: 0.31, green: 0.64, blue: 0.98)
    static let amber = Color(red: 0.98, green: 0.67, blue: 0.25)
    static let violet = Color(red: 0.65, green: 0.51, blue: 0.96)

    static let brandGradient = LinearGradient(
        colors: [coral, Color(red: 1.0, green: 0.58, blue: 0.32)],
        startPoint: .topLeading,
        endPoint: .bottomTrailing
    )
}

struct CleanerBackdrop: View {
    var body: some View {
        ZStack {
            Color(nsColor: .windowBackgroundColor)
            RadialGradient(
                colors: [CleanerTheme.azure.opacity(0.10), .clear],
                center: .topLeading,
                startRadius: 20,
                endRadius: 620
            )
            RadialGradient(
                colors: [CleanerTheme.coral.opacity(0.065), .clear],
                center: .bottomTrailing,
                startRadius: 30,
                endRadius: 520
            )
        }
        .accessibilityHidden(true)
    }
}

struct RootView: View {
    @EnvironmentObject private var model: AppModel
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        NavigationSplitView {
            sidebar
                .navigationSplitViewColumnWidth(min: 190, ideal: 218, max: 240)
        } detail: {
            ZStack {
                CleanerBackdrop()
                detail
            }
            .safeAreaInset(edge: .bottom, spacing: 0) { statusBar }
        }
        .tint(CleanerTheme.coral)
        .task { model.startMonitoring() }
        .background(
            WindowAppearanceConfigurator(isBusy: model.isBusy) {
                Task { await model.refreshStatus() }
            }
        )
    }

    private var sidebar: some View {
        ZStack {
            LinearGradient(
                colors: [
                    sidebarSurface,
                    sidebarSurface,
                    CleanerTheme.azure.opacity(colorScheme == .dark ? 0.045 : 0.025),
                    CleanerTheme.coral.opacity(colorScheme == .dark ? 0.035 : 0.018)
                ],
                startPoint: .top,
                endPoint: .bottom
            )

            VStack(alignment: .leading, spacing: 0) {
                HStack(spacing: 11) {
                    CleanerBrandMark()
                    .frame(width: 43, height: 43)
                    .shadow(color: CleanerTheme.coral.opacity(0.28), radius: 12, y: 6)

                    VStack(alignment: .leading, spacing: 1) {
                        Text("Mac Cleaner")
                            .font(.system(size: 15, weight: .semibold, design: .rounded))
                        Text("LOCAL · SAFE")
                            .font(.system(size: 9, weight: .bold, design: .rounded))
                            .tracking(1.25)
                            .foregroundStyle(CleanerTheme.mint)
                    }
                }
                .padding(.horizontal, 16)
                .padding(.top, 18)
                .padding(.bottom, 22)

                Text("WORKSPACE")
                    .font(.system(size: 9, weight: .bold, design: .rounded))
                    .tracking(1.4)
                    .foregroundStyle(.tertiary)
                    .padding(.horizontal, 18)
                    .padding(.bottom, 8)

                VStack(spacing: 5) {
                    ForEach(SidebarItem.allCases) { item in
                        SidebarButton(item: item, isSelected: (model.selection ?? .overview) == item) {
                            withAnimation(.spring(response: 0.28, dampingFraction: 0.82)) {
                                model.selection = item
                            }
                        }
                    }
                }
                .padding(.horizontal, 10)

                Spacer()

                HStack(spacing: 7) {
                    Circle()
                        .fill(model.error == nil ? CleanerTheme.mint : CleanerTheme.coral)
                        .frame(width: 7, height: 7)
                        .shadow(color: (model.error == nil ? CleanerTheme.mint : CleanerTheme.coral).opacity(0.55), radius: 5)
                    VStack(alignment: .leading, spacing: 1) {
                        Text(model.error == nil ? "Engine connected" : "Needs attention")
                            .font(.caption.weight(.medium))
                        Text("Approval required")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                }
                .padding(12)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(
                    colorScheme == .dark ? Color.white.opacity(0.055) : Color.white.opacity(0.72),
                    in: RoundedRectangle(cornerRadius: 18, style: .continuous)
                )
                .overlay {
                    RoundedRectangle(cornerRadius: 18, style: .continuous)
                        .stroke(
                            LinearGradient(
                                colors: [CleanerTheme.mint.opacity(0.24), CleanerTheme.azure.opacity(0.12)],
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            ),
                            lineWidth: 1
                        )
                }
                .padding(.horizontal, 12)
                .padding(.top, 12)
                .padding(.bottom, 38)
            }
        }
    }

    private var sidebarSurface: Color {
        colorScheme == .dark
            ? Color(red: 0.075, green: 0.085, blue: 0.095)
            : Color.white.opacity(0.97)
    }

    private var statusBar: some View {
        HStack(spacing: 9) {
            ZStack {
                Circle()
                    .fill((model.error == nil ? CleanerTheme.mint : CleanerTheme.coral).opacity(0.16))
                Image(systemName: model.error == nil ? "checkmark" : "exclamationmark")
                    .font(.system(size: 9, weight: .bold))
                    .foregroundStyle(model.error == nil ? CleanerTheme.mint : CleanerTheme.coral)
            }
            .frame(width: 20, height: 20)
            Text(model.error ?? model.notice)
                .font(.caption)
                .lineLimit(2)
            Spacer()
            Text("ON-DEVICE")
                .font(.system(size: 9, weight: .bold, design: .rounded))
                .tracking(1.1)
                .foregroundStyle(.tertiary)
            if model.error != nil {
                Button("Dismiss") { model.error = nil }
                    .buttonStyle(.plain)
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 7)
        .background(.ultraThinMaterial)
        .overlay(alignment: .top) { Divider().opacity(0.45) }
    }

    @ViewBuilder
    private var detail: some View {
        switch model.selection ?? .overview {
        case .overview: OverviewView()
        case .storage: StorageView()
        case .cleanup: CleanupView()
        case .quarantine: QuarantineView()
        case .activity: ActivityView()
        case .energy: EnergyView()
        case .autoClean: AutoCleanView()
        case .agents: AgentAccessView()
        }
    }
}

struct CleanerBrandMark: View {
    var body: some View {
        Group {
            if let url = Bundle.main.url(forResource: "MacCleanerMark", withExtension: "png"),
               let image = NSImage(contentsOf: url) {
                Image(nsImage: image)
                    .resizable()
                    .renderingMode(.original)
                    .scaledToFit()
            } else {
                Image(systemName: "externaldrive.fill.badge.checkmark")
                    .resizable()
                    .scaledToFit()
                    .foregroundStyle(CleanerTheme.coral)
                    .padding(5)
            }
        }
        .accessibilityLabel("Mac Cleaner")
    }
}

private struct WindowAppearanceConfigurator: NSViewRepresentable {
    let isBusy: Bool
    let refresh: () -> Void

    final class Coordinator {
        weak var window: NSWindow?
        var accessory: NSTitlebarAccessoryViewController?
        var hostingView: NSHostingView<TitlebarRefreshControl>?
    }

    func makeCoordinator() -> Coordinator { Coordinator() }

    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        configure(view, coordinator: context.coordinator)
        return view
    }

    func updateNSView(_ view: NSView, context: Context) {
        configure(view, coordinator: context.coordinator)
    }

    static func dismantleNSView(_ view: NSView, coordinator: Coordinator) {
        guard let window = coordinator.window,
              let accessory = coordinator.accessory,
              let index = window.titlebarAccessoryViewControllers.firstIndex(where: { $0 === accessory }) else { return }
        window.removeTitlebarAccessoryViewController(at: index)
    }

    private func configure(_ view: NSView, coordinator: Coordinator) {
        DispatchQueue.main.async {
            guard let window = view.window else { return }
            window.hasShadow = false

            let control = TitlebarRefreshControl(isBusy: isBusy, refresh: refresh)
            if let hostingView = coordinator.hostingView {
                hostingView.rootView = control
                return
            }

            let hostingView = NSHostingView(rootView: control)
            hostingView.frame = NSRect(x: 0, y: 0, width: 42, height: 34)

            let accessory = NSTitlebarAccessoryViewController()
            accessory.layoutAttribute = .right
            accessory.view = hostingView
            window.addTitlebarAccessoryViewController(accessory)

            coordinator.window = window
            coordinator.accessory = accessory
            coordinator.hostingView = hostingView
        }
    }
}

private struct TitlebarRefreshControl: View {
    let isBusy: Bool
    let refresh: () -> Void

    var body: some View {
        Button(action: refresh) {
            ZStack {
                Circle()
                    .fill(Color.primary.opacity(0.065))
                Image(systemName: "arrow.clockwise")
                    .opacity(isBusy ? 0 : 1)
                ProgressView()
                    .progressViewStyle(.circular)
                    .controlSize(.mini)
                    .tint(CleanerTheme.coral)
                    .opacity(isBusy ? 1 : 0)
            }
            .frame(width: 30, height: 30)
            .contentShape(Circle())
        }
        .buttonStyle(.plain)
        .disabled(isBusy)
        .accessibilityLabel(isBusy ? "Refreshing system status" : "Refresh system status")
        .help("Refresh system status")
    }
}

private struct SidebarButton: View {
    let item: SidebarItem
    let isSelected: Bool
    let action: () -> Void
    @State private var isHovering = false

    var body: some View {
        Button(action: action) {
            HStack(spacing: 11) {
                ZStack {
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .fill(isSelected ? CleanerTheme.coral.opacity(0.18) : Color.clear)
                    Image(systemName: item.symbol)
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundStyle(isSelected ? CleanerTheme.coral : .secondary)
                }
                .frame(width: 29, height: 29)

                Text(item.rawValue)
                    .font(.system(size: 13, weight: isSelected ? .semibold : .medium, design: .rounded))
                Spacer()
                if isSelected {
                    Capsule()
                        .fill(CleanerTheme.coral)
                        .frame(width: 3, height: 18)
                        .shadow(color: CleanerTheme.coral.opacity(0.5), radius: 5)
                }
            }
            .foregroundStyle(isSelected ? .primary : .secondary)
            .padding(.horizontal, 8)
            .padding(.vertical, 5)
            .background(
                Capsule(style: .continuous)
                    .fill(isSelected ? CleanerTheme.coral.opacity(0.095) : (isHovering ? Color.primary.opacity(0.04) : .clear))
            )
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { isHovering = $0 }
        .accessibilityAddTraits(isSelected ? .isSelected : [])
    }
}
