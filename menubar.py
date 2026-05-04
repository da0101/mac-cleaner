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
import webbrowser

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


# ── Server helpers (with standalone fallbacks) ───────────────────────────────

try:
    from mac_cleaner_server.memory import purge_ram as _do_purge
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


# ── App ──────────────────────────────────────────────────────────────────────

DASHBOARD_URL = "http://127.0.0.1:3333"
_INTERVAL = 5


class MemoryBarApp(rumps.App):
    def __init__(self) -> None:
        super().__init__("Mac Cleaner", title="… GB")
        self._stats = rumps.MenuItem("Loading…")
        self._detail = rumps.MenuItem("")
        self.menu = [
            self._stats,
            self._detail,
            None,
            rumps.MenuItem("Purge RAM", callback=self._purge),
            rumps.MenuItem("Open Dashboard", callback=self._open_dashboard),
        ]
        try:
            _apply_pill_bg(self._nsapp.nsstatusitem.button())
        except Exception:
            pass
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

        self._stats.title = (
            f"Free  {fmt_size(free)}   ·   Available  {fmt_size(avail)}   ·   Total  {fmt_size(total)}"
        )
        self._detail.title = (
            f"Wired  {fmt_size(wired)}   ·   Cached  {fmt_size(cached)}"
        )

    def _purge(self, _) -> None:
        self._stats.title = "Purging RAM…"
        ok = _do_purge()
        self._tick(None)
        if not ok:
            rumps.notification(
                "Mac Cleaner", "Purge RAM",
                "Could not purge — needs sudo access.",
            )

    def _open_dashboard(self, _) -> None:
        webbrowser.open(DASHBOARD_URL)


if __name__ == "__main__":
    MemoryBarApp().run()
