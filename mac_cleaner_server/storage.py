from __future__ import annotations

import subprocess
from datetime import datetime

from scanner import clean_scan_item, scan_known_items, scan_system_data_items

from .state import save_history, state
from .system import fmt_size

def scan_garbage():
    """Scan and return list of garbage items."""
    state["scanning"] = True
    try:
        items = scan_known_items()
        items.extend(scan_system_data_items())
        items.sort(key=lambda x: x["size"], reverse=True)
        state["last_scan"] = datetime.now().isoformat()
        state["last_storage_items"] = items
        return items
    finally:
        state["scanning"] = False


def clean_item(item):
    """Clean a single item. Returns bytes cleaned."""
    return clean_scan_item(item)


def clean_all_garbage():
    """Scan and clean all garbage. Returns summary."""
    state["cleaning"] = True
    items = [item for item in scan_known_items() if item.get("cleanable", False)]
    total_before = sum(i["size"] for i in items)
    cleaned = 0
    cleaned_items = []

    for item in items:
        freed = clean_item(item)
        if freed > 0:
            cleaned += freed
            cleaned_items.append({"name": item["name"], "freed": fmt_size(freed)})

    summary = {
        "timestamp": datetime.now().isoformat(),
        "items_found": len(items),
        "items_cleaned": len(cleaned_items),
        "total_garbage": fmt_size(total_before),
        "total_cleaned": fmt_size(cleaned),
        "total_cleaned_bytes": cleaned,
        "details": cleaned_items,
    }

    state["last_clean"] = summary["timestamp"]
    state["history"].insert(0, summary)
    save_history()
    state["cleaning"] = False
    return summary


def docker_prune():
    """Run docker system prune."""
    try:
        result = subprocess.run(
            ["docker", "system", "prune", "-a", "-f", "--volumes"],
            capture_output=True, text=True, timeout=120
        )
        return {"success": result.returncode == 0, "output": result.stdout[-500:] if result.stdout else ""}
    except Exception as e:
        return {"success": False, "output": str(e)}
