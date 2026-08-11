interface Props {
  launchAtLogin: boolean;
  loginItemAvailable: boolean;
  onLaunchAtLoginChange: (enabled: boolean) => Promise<void>;
}

export function AppHeader({ launchAtLogin, loginItemAvailable, onLaunchAtLoginChange }: Props) {
  return <header className="topbar">
    <div className="brand">
      <span className="brand-mark"><img src="/app-icon.png" alt="" /></span>
      <div><strong>Mac Cleaner</strong><span>Local Rust utility</span></div>
    </div>
    <label className="login-toggle">
      <span>Launch at login</span>
      <input
        type="checkbox"
        checked={launchAtLogin}
        disabled={!loginItemAvailable}
        title={loginItemAvailable ? "" : "Available after installing Mac Cleaner.app"}
        onChange={(event) => void onLaunchAtLoginChange(event.target.checked)}
      />
      {!loginItemAvailable && <small>Installed app only</small>}
    </label>
  </header>;
}
