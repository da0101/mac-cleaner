import SwiftUI

struct NativeEmptyState<Actions: View>: View {
    let title: String
    let symbol: String
    let description: String
    @ViewBuilder let actions: Actions

    init(
        _ title: String,
        systemImage: String,
        description: String = "",
        @ViewBuilder actions: () -> Actions
    ) {
        self.title = title
        symbol = systemImage
        self.description = description
        self.actions = actions()
    }

    var body: some View {
        VStack(spacing: 13) {
            ZStack {
                RoundedRectangle(cornerRadius: 18, style: .continuous)
                    .fill(CleanerTheme.coral.opacity(0.12))
                Image(systemName: symbol)
                    .font(.system(size: 27, weight: .semibold))
                    .foregroundStyle(CleanerTheme.coral)
            }
            .frame(width: 64, height: 64)
            Text(title).font(.system(.title3, design: .rounded, weight: .semibold))
            if !description.isEmpty {
                Text(description).foregroundStyle(.secondary).multilineTextAlignment(.center)
            }
            actions
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding(32)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 20, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 20, style: .continuous)
                .stroke(Color.primary.opacity(0.08), lineWidth: 1)
        }
    }
}

extension NativeEmptyState where Actions == EmptyView {
    init(_ title: String, systemImage: String, description: String = "") {
        self.init(title, systemImage: systemImage, description: description) { EmptyView() }
    }
}
