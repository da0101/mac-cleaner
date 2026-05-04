#!/usr/bin/env python3
"""macOS menu bar widget — RAM chip with fill bar + rich attributed title.

Visual layout:
  [ chip░░░░████ ]  4.2  /  11.6 GB
   ─────────────   ───     ────
   drawn fill bar  bold    semibold white
   pressure color  color   (more visible)

Updates every 5 s.  Reuses mac_cleaner_server helpers when run from the
project root; falls back to inline vm_stat parsing otherwise.
"""
from __future__ import annotations

import subprocess
import sys
import time
import urllib.request
import webbrowser
import json

try:
    import rumps
except ImportError:
    print("rumps is required.  Run:  pip install rumps", file=sys.stderr)
    sys.exit(1)

from AppKit import (  # type: ignore[import]
    NSAttributedString,
    NSMutableAttributedString,
    NSBezierPath,
    NSColor,
    NSFont,
    NSFontAttributeName,
    NSForegroundColorAttributeName,
    NSBaselineOffsetAttributeName,
    NSImage,
    NSTextAttachment,
)

try:
    from Foundation import NSMakeRect, NSMakeSize  # type: ignore[import]
except ImportError:
    def NSMakeRect(x, y, w, h): return ((x, y), (w, h))   # type: ignore[misc]
    def NSMakeSize(w, h):        return (w, h)              # type: ignore[misc]


# ── Pressure colors ─────────────────────────────────────────────────────────

_GB = 1024 ** 3


def _ram_color(free_bytes: int) -> NSColor:
    if free_bytes >= 4 * _GB:
        return NSColor.systemGreenColor()
    if free_bytes >= 2 * _GB:
        return NSColor.systemOrangeColor()
    return NSColor.systemRedColor()


# ── Custom chip icon ─────────────────────────────────────────────────────────

def _draw_chip(fill_ratio: float, fill_color: NSColor) -> NSImage:
    """
    Draw a vertical RAM module with a proportional fill bar inside.
    """
    W, H = 14.0, 16.0
    image = NSImage.alloc().initWithSize_(NSMakeSize(W, H))
    image.lockFocus()
    try:
        label  = NSColor.labelColor()
        subtle = NSColor.colorWithWhite_alpha_(0.5, 0.13)

        body = NSBezierPath.bezierPathWithRoundedRect_xRadius_yRadius_(
            NSMakeRect(2.2, 0.8, W - 4.0, H - 1.6), 1.8, 1.8
        )
        subtle.setFill()
        body.fill()

        pad    = 1.5
        max_fh = H - 1.6 - pad * 2
        fh     = max(0.0, max_fh * min(max(fill_ratio, 0.0), 1.0))
        if fh > 0.8:
            fill_bar = NSBezierPath.bezierPathWithRoundedRect_xRadius_yRadius_(
                NSMakeRect(2.2 + pad, 0.8 + pad, W - 4.0 - pad * 2, fh),
                1.2, 1.2,
            )
            fill_color.colorWithAlphaComponent_(0.95).setFill()
            fill_bar.fill()

        label.colorWithAlphaComponent_(0.65).setStroke()
        body.setLineWidth_(1.0)
        body.stroke()

        contacts = NSBezierPath.bezierPath()
        contacts.setLineWidth_(0.9)
        for py in (H * 0.25, H * 0.42, H * 0.59, H * 0.76):
            contacts.moveToPoint_((2.0, py))
            contacts.lineToPoint_((0.7, py))
        label.colorWithAlphaComponent_(0.65).setStroke()
        contacts.stroke()

    except Exception:
        pass
    finally:
        image.unlockFocus()
    return image


# ── Pill background (translucent frosted) ────────────────────────────────────

def _apply_pill_bg(button) -> None:
    try:
        button.setWantsLayer_(True)
        layer = button.layer()
        layer.setBackgroundColor_(NSColor.colorWithWhite_alpha_(1.0, 0.09).CGColor())
        layer.setCornerRadius_(9.0)
        layer.setBorderColor_(NSColor.colorWithWhite_alpha_(1.0, 0.14).CGColor())
        layer.setBorderWidth_(0.5)
    except Exception:
        pass


# ── Rich attributed title ────────────────────────────────────────────────────

def _num(b: int) -> str:
    return f"{b / _GB:.1f} GB" if b >= _GB else f"{b / 1024 ** 2:.0f} MB"


def _build_title(free: int, avail: int) -> NSMutableAttributedString:
    color      = _ram_color(free)
    faint      = NSColor.colorWithWhite_alpha_(1.0, 0.75)
    fill_ratio = (free / avail) if avail > 0 else 0.0

    bold_font  = NSFont.monospacedDigitSystemFontOfSize_weight_(13.0, 0.62)  # black
    sep_font   = NSFont.systemFontOfSize_(11.0)
    avail_font = NSFont.monospacedDigitSystemFontOfSize_weight_(12.0, 0.30)  # semibold
    label_font = NSFont.systemFontOfSize_(10.0)

    result = NSMutableAttributedString.alloc().init()

    # ── Chip icon with fill bar ───────────────────────────────────────────
    try:
        chip = _draw_chip(fill_ratio, color)
        att  = NSTextAttachment.alloc().init()
        att.setImage_(chip)
        try:
            att.setBounds_(NSMakeRect(0, -3.0, 14.0, 16.0))
        except Exception:
            pass
        sym = NSMutableAttributedString.alloc().initWithAttributedString_(
            NSAttributedString.attributedStringWithAttachment_(att)
        )
        sym.addAttribute_value_range_(NSBaselineOffsetAttributeName, -1.5, (0, sym.length()))
        result.appendAttributedString_(sym)
        # thin gap
        result.appendAttributedString_(
            NSAttributedString.alloc().initWithString_attributes_(
                " ", {NSFontAttributeName: NSFont.systemFontOfSize_(4.0)}
            )
        )
    except Exception:
        pass

    # ── Free RAM — bold, pressure color ──────────────────────────────────
    result.appendAttributedString_(
        NSAttributedString.alloc().initWithString_attributes_(
            _num(free),
            {NSForegroundColorAttributeName: color, NSFontAttributeName: bold_font},
        )
    )

    # ── "Free" label ──────────────────────────────────────────────────────
    result.appendAttributedString_(
        NSAttributedString.alloc().initWithString_attributes_(
            " Free",
            {NSForegroundColorAttributeName: faint, NSFontAttributeName: label_font},
        )
    )

    # ── Separator — visible ───────────────────────────────────────────────
    result.appendAttributedString_(
        NSAttributedString.alloc().initWithString_attributes_(
            " / ",
            {NSForegroundColorAttributeName: NSColor.secondaryLabelColor(), NSFontAttributeName: sep_font},
        )
    )

    # ── Available RAM — semibold, bright ──────────────────────────────────
    result.appendAttributedString_(
        NSAttributedString.alloc().initWithString_attributes_(
            _num(avail),
            {
                NSForegroundColorAttributeName: NSColor.labelColor(),
                NSFontAttributeName: avail_font,
            },
        )
    )

    return result


def _apply_title(app: rumps.App, free: int, avail: int) -> None:
    try:
        app._nsapp.nsstatusitem.button().setAttributedTitle_(_build_title(free, avail))
    except Exception:
        app.title = f"{_num(free)} Free / {_num(avail)}"


def _set_menu_title(item: rumps.MenuItem, title: str, color: NSColor | None = None, bold: bool = False, size: float = 13.0) -> None:
    item.title = title
    try:
        item._menuitem.setEnabled_(True)
        font = NSFont.monospacedDigitSystemFontOfSize_weight_(size, 0.62 if bold else 0.32)
        attrs = {NSFontAttributeName: font}
        if color is not None:
            attrs[NSForegroundColorAttributeName] = color
        item._menuitem.setAttributedTitle_(NSAttributedString.alloc().initWithString_attributes_(title, attrs))
    except Exception:
        pass


def _muted_color() -> NSColor:
    return NSColor.colorWithCalibratedRed_green_blue_alpha_(0.62, 0.68, 0.74, 1.0)


def _primary_text_color() -> NSColor:
    return NSColor.colorWithCalibratedRed_green_blue_alpha_(0.9, 0.94, 0.98, 1.0)


def _garbage_color(bytes_count: int) -> NSColor:
    if bytes_count >= _GB:
        return NSColor.systemRedColor()
    if bytes_count >= 250 * 1024 ** 2:
        return NSColor.systemOrangeColor()
    return NSColor.systemGreenColor()


def _process_color(bytes_count: int, can_close: bool) -> NSColor:
    if not can_close:
        return _muted_color()
    if bytes_count >= _GB:
        return NSColor.systemRedColor()
    if bytes_count >= 500 * 1024 ** 2:
        return NSColor.systemOrangeColor()
    return NSColor.labelColor()


# ── Server helpers (with standalone fallbacks) ───────────────────────────────

try:
    from mac_cleaner_server.memory import purge_ram as _do_purge
    from mac_cleaner_server.processes import get_top_memory_processes
    from mac_cleaner_server.state import DEFAULT_SETTINGS, setting
    from mac_cleaner_server.storage import clean_all_garbage, docker_prune, scan_garbage
    from mac_cleaner_server.system import fmt_size, get_system_info
except ImportError:
    def get_system_info() -> dict:  # type: ignore[misc]
        info: dict = {"free_ram": 0, "available_ram": 0, "total_ram": 0,
                      "ram_wired": 0, "ram_cached": 0}
        try:
            out = subprocess.run(["vm_stat"], capture_output=True, text=True, timeout=5)
            if out.returncode != 0:
                return info
            pages: dict[str, int] = {}
            for line in out.stdout.strip().split("\n")[1:]:
                if ":" in line:
                    k, v = line.split(":", 1)
                    try:
                        pages[k.strip()] = int(v.strip().rstrip("."))
                    except ValueError:
                        pass
            ps = 16384
            free        = pages.get("Pages free", 0) * ps
            active      = pages.get("Pages active", 0) * ps
            inactive    = pages.get("Pages inactive", 0) * ps
            wired       = pages.get("Pages wired down", 0) * ps
            compressed  = pages.get("Pages occupied by compressor", 0) * ps
            speculative = pages.get("Pages speculative", 0) * ps
            purgeable   = pages.get("Pages purgeable", 0) * ps
            info["free_ram"]      = free
            info["available_ram"] = free + inactive + speculative + purgeable
            info["total_ram"]     = free + active + inactive + wired + compressed + speculative
            info["ram_wired"]     = wired
            info["ram_cached"]    = inactive
        except Exception:
            pass
        return info

    def fmt_size(b: int) -> str:  # type: ignore[misc]
        if b >= 1024 ** 3:
            return f"{b / 1024 ** 3:.2f} GB"
        return f"{b / 1024 ** 2:.0f} MB"

    def _do_purge() -> bool:  # type: ignore[misc]
        try:
            r = subprocess.run(["purge"], capture_output=True, timeout=15)
            if r.returncode == 0:
                return True
            r = subprocess.run(["sudo", "-n", "purge"], capture_output=True, timeout=15)
            return r.returncode == 0
        except Exception:
            return False

    DEFAULT_SETTINGS = {"ram_purge_interval_seconds": 300}  # type: ignore[misc]

    def setting(name: str) -> int:  # type: ignore[misc]
        return DEFAULT_SETTINGS.get(name, 300)

    def get_top_memory_processes(limit=5) -> list:  # type: ignore[misc]
        return []

    def scan_garbage() -> list:  # type: ignore[misc]
        return []

    def clean_all_garbage() -> dict:  # type: ignore[misc]
        return {"items_cleaned": 0, "total_cleaned": "0 B"}

    def docker_prune() -> dict:  # type: ignore[misc]
        return {"success": False, "output": "Docker prune is unavailable."}


# ── App ──────────────────────────────────────────────────────────────────────

DASHBOARD_URL = "http://127.0.0.1:3333"
DASHBOARD_URLS = (DASHBOARD_URL, "http://localhost:3333")
_INTERVAL = 5
_STORAGE_SCAN_INTERVAL = 5 * 60


def _mmss(seconds: int) -> str:
    seconds = max(0, int(seconds))
    return f"{seconds // 60}:{seconds % 60:02d}"


def _dashboard_get(path: str, timeout: float = 2.0) -> dict:
    for base_url in DASHBOARD_URLS:
        try:
            with urllib.request.urlopen(f"{base_url}{path}", timeout=timeout) as response:
                return json_loads(response.read().decode("utf-8"))
        except Exception:
            continue
    return {}


def _dashboard_status() -> dict:
    return _dashboard_get("/api/auto-clean-status") or _dashboard_get("/api/status")


def _dashboard_post(path: str, payload: dict | None = None) -> dict:
    body = json.dumps(payload).encode("utf-8") if payload is not None else None
    headers = {"Content-Type": "application/json"} if payload is not None else {}
    for base_url in DASHBOARD_URLS:
        try:
            request = urllib.request.Request(f"{base_url}{path}", data=body, headers=headers, method="POST")
            with urllib.request.urlopen(request, timeout=4.0) as response:
                return json_loads(response.read().decode("utf-8"))
        except Exception:
            continue
    return {}


def json_loads(raw: str) -> dict:
    import json
    try:
        data = json.loads(raw)
    except Exception:
        return {}
    return data if isinstance(data, dict) else {}


def _confirm(title: str, message: str, ok: str) -> bool:
    try:
        return rumps.alert(title=title, message=message, ok=ok, cancel="Cancel") == 1
    except Exception:
        return False


class MemoryBarApp(rumps.App):
    def __init__(self) -> None:
        super().__init__("Mac Cleaner", title="… GB")
        self._stats = rumps.MenuItem("Loading…")
        self._detail = rumps.MenuItem("")
        self._memory_header = rumps.MenuItem("Memory")
        self._cleanup_header = rumps.MenuItem("Cleanup")
        self._garbage = rumps.MenuItem("Garbage: scanning…")
        self._ram_timer = rumps.MenuItem("RAM purge: checking…")
        self._clean_timer = rumps.MenuItem("Garbage clean: checking…")
        self._top_title = rumps.MenuItem("Top memory consumers")
        self._top_items = [rumps.MenuItem("Loading…") for _ in range(5)]
        self._toggle_auto = rumps.MenuItem("Enable Auto-Clean", callback=self._toggle_auto_clean)
        self._purge_item = rumps.MenuItem("Purge RAM", callback=self._purge)
        self._clean_item = rumps.MenuItem("Clean Garbage", callback=self._clean_garbage)
        self._docker_item = rumps.MenuItem("Docker Prune", callback=self._docker_prune)
        self._refresh_item = rumps.MenuItem("Refresh Stats", callback=self._refresh_now)
        self._dashboard_item = rumps.MenuItem("Open Dashboard", callback=self._open_dashboard)
        self._last_storage_scan = 0.0
        self._garbage_total = 0
        self._garbage_count = 0
        self._report_only_count = 0
        self._ram_purge_interval = int(setting("ram_purge_interval_seconds") or 300)
        self._garbage_clean_interval = int(setting("auto_clean_interval_seconds") or 900)
        self._next_ram_purge_at = time.time() + self._ram_purge_interval
        self._next_garbage_clean_at = time.time() + self._garbage_clean_interval
        self._auto_clean_enabled = False
        self._dashboard_connected = False
        self.menu = [
            self._memory_header,
            self._stats,
            self._detail,
            None,
            self._cleanup_header,
            self._garbage,
            self._ram_timer,
            self._clean_timer,
            None,
            self._top_title,
            *self._top_items,
            None,
            self._toggle_auto,
            self._purge_item,
            self._clean_item,
            self._docker_item,
            self._refresh_item,
            self._dashboard_item,
        ]
        try:
            _apply_pill_bg(self._nsapp.nsstatusitem.button())
        except Exception:
            pass
        self._style_static_menu_items()
        self._tick(None)

    @rumps.timer(_INTERVAL)
    def _tick(self, _) -> None:
        info   = get_system_info()
        free   = info.get("free_ram", 0)
        avail  = info.get("available_ram", 0)
        total  = info.get("total_ram", 0)
        wired  = info.get("ram_wired", 0)
        cached = info.get("ram_cached", info.get("ram_inactive", 0))

        _apply_title(self, free, avail)

        _set_menu_title(
            self._stats,
            f"Free {fmt_size(free)}   Available {fmt_size(avail)}   Total {fmt_size(total)}",
            _ram_color(free),
            bold=True,
        )
        _set_menu_title(
            self._detail,
            f"Wired {fmt_size(wired)}   Cached {fmt_size(cached)}",
            _primary_text_color(),
        )
        self._refresh_dashboard_timer()
        self._refresh_processes()
        if time.time() - self._last_storage_scan >= _STORAGE_SCAN_INTERVAL:
            self._refresh_storage()

    def _purge(self, _) -> None:
        if not _confirm("Purge RAM?", "Run macOS memory purge now? Apps stay open, but the system may pause briefly.", "Purge RAM"):
            return
        self._stats.title = "Purging RAM…"
        result = _dashboard_post("/api/purge-ram")
        ok = bool(result.get("success")) if result else _do_purge()
        self._tick(None)
        if not ok:
            rumps.notification(
                "Mac Cleaner", "Purge RAM",
                "Could not purge — needs sudo access.",
            )
        else:
            rumps.notification("Mac Cleaner", "Purge RAM", "RAM purge completed.")

    def _clean_garbage(self, _) -> None:
        items = scan_garbage()
        cleanable = [item for item in items if item.get("cleanable", False)]
        total = sum(item.get("size", 0) for item in cleanable)
        if not cleanable:
            self._set_garbage(items)
            rumps.notification("Mac Cleaner", "Clean Garbage", "No cleanable garbage found.")
            return
        if not _confirm(
            "Clean garbage?",
            f"Clean {len(cleanable)} scanner-approved item(s), totaling {fmt_size(total)}?\n\nReport-only and protected paths stay untouched.",
            "Clean",
        ):
            return
        summary = _dashboard_post("/api/clean") or clean_all_garbage()
        self._last_storage_scan = 0
        self._refresh_storage()
        self._refresh_dashboard_timer()
        rumps.notification(
            "Mac Cleaner",
            "Clean Garbage",
            f"Cleaned {summary.get('total_cleaned', '0 B')} from {summary.get('items_cleaned', 0)} item(s).",
        )

    def _docker_prune(self, _) -> None:
        if not _confirm(
            "Docker prune?",
            "Run Docker system prune with volumes? This can remove unused images, containers, build cache, and volumes.",
            "Prune",
        ):
            return
        self._garbage.title = "Docker prune: running…"
        result = _dashboard_post("/api/docker-prune") or docker_prune()
        if result.get("success"):
            rumps.notification("Mac Cleaner", "Docker Prune", "Docker prune completed.")
        else:
            rumps.notification("Mac Cleaner", "Docker Prune", result.get("output", "Docker prune failed.")[:160])
        self._last_storage_scan = 0
        self._refresh_storage()

    def _refresh_now(self, _) -> None:
        self._last_storage_scan = 0
        self._tick(None)

    def _open_dashboard(self, _) -> None:
        webbrowser.open(DASHBOARD_URL)

    def _refresh_dashboard_timer(self) -> None:
        status = _dashboard_status()
        self._dashboard_connected = bool(status)
        if status:
            settings = status.get("settings") if isinstance(status.get("settings"), dict) else {}
            interval = int(settings.get("ram_purge_interval_seconds") or self._ram_purge_interval or 300)
            if interval != self._ram_purge_interval:
                self._ram_purge_interval = interval
            clean_interval = int(settings.get("auto_clean_interval_seconds") or self._garbage_clean_interval or 900)
            if clean_interval != self._garbage_clean_interval:
                self._garbage_clean_interval = clean_interval
            if "auto_clean_enabled" in status:
                self._auto_clean_enabled = bool(status.get("auto_clean_enabled"))
            elif "enabled" in status:
                self._auto_clean_enabled = bool(status.get("enabled"))
            server_time = float(status.get("server_time") or time.time())
            if status.get("next_ram_purge"):
                self._next_ram_purge_at = time.time() + max(0, float(status["next_ram_purge"]) - server_time)
            elif not self._auto_clean_enabled:
                self._next_ram_purge_at = time.time() + self._ram_purge_interval
            if status.get("next_garbage_clean"):
                self._next_garbage_clean_at = time.time() + max(0, float(status["next_garbage_clean"]) - server_time)
            elif not self._auto_clean_enabled:
                self._next_garbage_clean_at = time.time() + self._garbage_clean_interval
        now = time.time()
        ram_remaining = int(self._next_ram_purge_at - now)
        if ram_remaining <= 0:
            self._next_ram_purge_at = time.time() + self._ram_purge_interval
            ram_remaining = self._ram_purge_interval
        clean_remaining = int(self._next_garbage_clean_at - now)
        if clean_remaining <= 0:
            self._next_garbage_clean_at = time.time() + self._garbage_clean_interval
            clean_remaining = self._garbage_clean_interval
        suffix = "" if self._auto_clean_enabled else "  paused"
        if self._auto_clean_enabled and not self._dashboard_connected:
            suffix = "  syncing"
        color = NSColor.systemGreenColor() if self._auto_clean_enabled else NSColor.systemRedColor()
        _set_menu_title(self._ram_timer, f"Next RAM purge:     {_mmss(ram_remaining)}{suffix}", color, bold=self._auto_clean_enabled)
        _set_menu_title(self._clean_timer, f"Next garbage clean: {_mmss(clean_remaining)}{suffix}", color, bold=self._auto_clean_enabled)
        _set_menu_title(
            self._toggle_auto,
            "Pause Auto-Clean" if self._auto_clean_enabled else "Enable Auto-Clean",
            NSColor.systemOrangeColor() if self._auto_clean_enabled else NSColor.systemGreenColor(),
            bold=True,
        )

    def _refresh_processes(self) -> None:
        procs = get_top_memory_processes(5)
        if not procs:
            self._top_items[0].title = "No process data available"
            for item in self._top_items[1:]:
                item.title = ""
            return
        for idx, item in enumerate(self._top_items):
            if idx >= len(procs):
                item.title = ""
                continue
            proc = procs[idx]
            count = proc.get("count", 0)
            count_label = f" x{count}" if count and count > 1 else ""
            protected = "" if proc.get("can_close") else " · protected"
            label = f"{idx + 1}. {proc.get('label', 'Process')}{count_label:<4} {proc.get('mem', '0 B')}{protected}"
            _set_menu_title(item, label, _process_color(proc.get("mem_bytes", 0), bool(proc.get("can_close"))), bold=idx == 0)

    def _refresh_storage(self) -> None:
        items = scan_garbage()
        self._set_garbage(items)

    def _set_garbage(self, items: list) -> None:
        cleanable = [item for item in items if item.get("cleanable", False)]
        self._garbage_total = sum(item.get("size", 0) for item in cleanable)
        self._garbage_count = len(cleanable)
        self._report_only_count = len(items) - len(cleanable)
        self._last_storage_scan = time.time()
        _set_menu_title(
            self._garbage,
            f"Garbage: {fmt_size(self._garbage_total)}   {self._garbage_count} cleanable   {self._report_only_count} report-only",
            _garbage_color(self._garbage_total),
            bold=self._garbage_total > 0,
        )

    def _style_static_menu_items(self) -> None:
        _set_menu_title(self._memory_header, "Memory", NSColor.systemBlueColor(), bold=True, size=12.0)
        _set_menu_title(self._cleanup_header, "Cleanup", NSColor.systemOrangeColor(), bold=True, size=12.0)
        _set_menu_title(self._top_title, "Top memory consumers", NSColor.systemPurpleColor(), bold=True, size=12.0)
        _set_menu_title(self._toggle_auto, "Enable Auto-Clean", NSColor.systemGreenColor(), bold=True)
        _set_menu_title(self._purge_item, "Purge RAM", NSColor.systemBlueColor(), bold=True)
        _set_menu_title(self._clean_item, "Clean Garbage", NSColor.systemOrangeColor(), bold=True)
        _set_menu_title(
            self._docker_item,
            "Docker Prune",
            NSColor.colorWithCalibratedRed_green_blue_alpha_(0.2, 0.8, 0.72, 1.0),
            bold=True,
        )
        _set_menu_title(self._refresh_item, "Refresh Stats", _primary_text_color(), bold=True)
        _set_menu_title(self._dashboard_item, "Open Dashboard", _primary_text_color(), bold=True)

    def _toggle_auto_clean(self, _) -> None:
        if not _confirm(
            "Auto-Clean",
            f"{'Pause' if self._auto_clean_enabled else 'Enable'} automatic RAM purge and garbage cleanup?",
            "Pause" if self._auto_clean_enabled else "Enable",
        ):
            return
        target = not self._auto_clean_enabled
        result = _dashboard_post("/api/auto-clean", {"enabled": target})
        if not result:
            rumps.notification("Mac Cleaner", "Auto-Clean", "Dashboard server is not reachable.")
            return
        self._auto_clean_enabled = bool(result.get("auto_clean_enabled", result.get("enabled")))
        self._dashboard_connected = True
        if result.get("next_ram_purge") and result.get("server_time"):
            self._next_ram_purge_at = time.time() + max(0, float(result["next_ram_purge"]) - float(result["server_time"]))
        if result.get("next_garbage_clean") and result.get("server_time"):
            self._next_garbage_clean_at = time.time() + max(0, float(result["next_garbage_clean"]) - float(result["server_time"]))
        self._refresh_dashboard_timer()
        rumps.notification("Mac Cleaner", "Auto-Clean", "Enabled" if self._auto_clean_enabled else "Paused")


if __name__ == "__main__":
    MemoryBarApp().run()
