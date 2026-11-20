#!/usr/bin/env python3
"""Monitor layout & DDC brightness helper for niri, using switchmon layout definitions."""

from concurrent.futures import ThreadPoolExecutor
import json
import re
import subprocess
import sys

FUZZEL_CONFIG = "/home/arhammahajan/dev/dots/fuzzel/monitor-layout.ini"

# Layout definitions adapted from ~/.local/bin/switchmon for niri.
# Mutter transform 2 (upside-down) -> niri "180"
# Mutter transform 3 (clockwise portrait) -> niri "270"
LAYOUTS = {
    "original": {
        "AW3426DWM": {
            "mode": "3440x1440@239.984",
            "vrr": "on",
            "x": 0,
            "y": 216,
            "scale": "1.0",
            "transform": "normal",
        },
        "BenQ GW2480": {
            "mode": "1920x1080@60.000",
            "vrr": None,
            "x": 3440,
            "y": 0,
            "scale": "1.0",
            "transform": "270",
        },
    },
    "stacked": {
        "BenQ GW2270": {
            "mode": "1920x1080@60.000",
            "vrr": None,
            "x": 0,
            "y": 0,
            "scale": "1.0",
            "transform": "180",
        },
        "LG ULTRAGEAR": {
            "mode": "1920x1080@143.981",
            "vrr": None,
            "x": 0,
            "y": 1080,
            "scale": "1.0",
            "transform": "normal",
        },
    },
    "all": {
        "AW3426DWM": {"mode": "3440x1440@239.984", "vrr": "on", "scale": "1.0", "transform": "normal"},
        "BenQ GW2480": {"mode": "1920x1080@60.000", "vrr": None, "scale": "1.0", "transform": "normal"},
        "BenQ GW2270": {"mode": "1920x1080@60.000", "vrr": None, "scale": "1.0", "transform": "normal"},
        "LG ULTRAGEAR": {"mode": "1920x1080@143.981", "vrr": None, "scale": "1.0", "transform": "normal"},
    },
}

LAYOUT_DESCRIPTIONS = {
    "original": "AW3426DWM + GW2480 (portrait)",
    "stacked": "GW2270 (180°) + ULTRAGEAR",
    "all": "All 4 monitors",
}


def notify(title: str, body: str, urgency: str = "normal") -> None:
    subprocess.run(
        ["notify-send", "-u", urgency, title, body],
        check=False,
    )


def get_outputs() -> dict:
    res = subprocess.run(
        ["niri", "msg", "-j", "outputs"],
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(res.stdout)


def detect_active_layout(outputs: dict) -> str | None:
    active_models = {
        info.get("model")
        for info in outputs.values()
        if info.get("logical") is not None and info.get("model")
    }
    for name, spec in LAYOUTS.items():
        if active_models == set(spec.keys()):
            return name
    return None


def detect_ddc_buses() -> dict[str, int]:
    """Return mapping of DRM connector (e.g. 'DP-3') -> I2C bus number for valid DDC displays."""
    res = subprocess.run(
        ["ddcutil", "detect", "--brief"],
        capture_output=True,
        text=True,
        check=False,
    )
    buses: dict[str, int] = {}
    for block in res.stdout.strip().split("\n\n"):
        lines = block.splitlines()
        if not lines or not lines[0].startswith("Display "):
            continue
        bus_match = re.search(r"I2C bus:\s+/dev/i2c-(\d+)", block)
        conn_match = re.search(r"DRM connector:\s+\S+?-([A-Za-z0-9-]+)", block)
        if bus_match and conn_match:
            buses[conn_match.group(1)] = int(bus_match.group(1))
    return buses


def get_ddc_brightness(bus: int) -> tuple[int, int] | None:
    res = subprocess.run(
        ["ddcutil", "getvcp", "10", "--bus", str(bus), "--terse"],
        capture_output=True,
        text=True,
        check=False,
    )
    if res.returncode != 0:
        return None
    # Format: "VCP 10 C <current> <max>"
    parts = res.stdout.strip().split()
    if len(parts) >= 5 and parts[0] == "VCP":
        try:
            return int(parts[3]), int(parts[4])
        except ValueError:
            return None
    return None


def set_ddc_brightness(bus: int, value: int) -> None:
    clamped = max(0, min(100, value))
    subprocess.run(
        ["ddcutil", "setvcp", "10", str(clamped), "--bus", str(bus), "--noverify"],
        check=True,
        capture_output=True,
        text=True,
    )


def get_status() -> dict:
    outputs = get_outputs()
    active_layout = detect_active_layout(outputs)
    connected_models = {info.get("model") for info in outputs.values() if info.get("model")}

    ddc_buses = detect_ddc_buses()
    brightness_map: dict[int, tuple[int, int] | None] = {}
    if ddc_buses:
        with ThreadPoolExecutor(max_workers=len(ddc_buses)) as pool:
            futures = {bus: pool.submit(get_ddc_brightness, bus) for bus in ddc_buses.values()}
            for bus, fut in futures.items():
                brightness_map[bus] = fut.result()

    monitors = []
    for connector, info in sorted(outputs.items()):
        model = info.get("model") or connector
        bus = ddc_buses.get(connector)
        bright = brightness_map.get(bus) if bus is not None else None
        monitors.append({
            "model": model,
            "connector": connector,
            "active": info.get("logical") is not None,
            "bus": bus,
            "brightness": bright[0] if bright else None,
            "max_brightness": bright[1] if bright else None,
        })

    layouts = [
        {
            "id": name,
            "label": name,
            "desc": LAYOUT_DESCRIPTIONS.get(name, ""),
            "available": set(spec.keys()).issubset(connected_models),
        }
        for name, spec in LAYOUTS.items()
    ]

    return {
        "active_layout": active_layout,
        "layouts": layouts,
        "monitors": monitors,
    }


def niri_output(connector: str, *args: str) -> None:
    subprocess.run(
        ["niri", "msg", "output", connector, *args],
        check=True,
        capture_output=True,
        text=True,
    )


def apply_layout(name: str, outputs: dict) -> None:
    if name not in LAYOUTS:
        raise ValueError(f"Unknown layout: {name}")

    by_model = {}
    for connector, info in outputs.items():
        model = info.get("model")
        if model:
            by_model[model] = connector

    target = LAYOUTS[name]
    missing = [m for m in target if m not in by_model]
    if missing:
        raise RuntimeError(f"Missing monitor(s) for '{name}': {', '.join(missing)}")

    # Enable and configure target monitors first so niri always has an active output.
    for model, cfg in target.items():
        connector = by_model[model]
        niri_output(connector, "on")
        niri_output(connector, "mode", cfg["mode"])
        niri_output(connector, "scale", cfg["scale"])
        niri_output(connector, "transform", cfg["transform"])
        if "x" in cfg and "y" in cfg:
            niri_output(connector, "position", "set", str(cfg["x"]), str(cfg["y"]))
        else:
            niri_output(connector, "position", "auto")
        if cfg.get("vrr") is not None:
            niri_output(connector, "vrr", cfg["vrr"])

    # Disable monitors not in the target layout.
    for model, connector in by_model.items():
        if model not in target:
            niri_output(connector, "off")


def pick_via_fuzzel(active: str | None) -> str | None:
    labels = {}
    lines = []
    for name in LAYOUTS:
        label = f"{name} (active)" if name == active else name
        labels[label] = name
        lines.append(label)

    res = subprocess.run(
        [
            "fuzzel",
            "--dmenu",
            "--lines",
            str(len(lines)),
            "-w",
            "26",
            "--config",
            FUZZEL_CONFIG,
        ],
        input="\n".join(lines),
        capture_output=True,
        text=True,
    )
    if res.returncode != 0:
        return None
    choice = res.stdout.strip()
    return labels.get(choice)


def main() -> None:
    args = sys.argv[1:]
    if args == ["--status"]:
        print(json.dumps(get_status()))
        return
    if len(args) == 3 and args[0] == "--set-brightness":
        set_ddc_brightness(int(args[1]), int(args[2]))
        return

    outputs = get_outputs()
    active = detect_active_layout(outputs)

    if args:
        arg = args[0]
        if arg == "toggle":
            choice = "original" if active == "stacked" else "stacked"
        else:
            choice = arg
    else:
        choice = pick_via_fuzzel(active)
        if not choice:
            return

    apply_layout(choice, outputs)
    notify("Monitor Layout", f"Switched to '{choice}'")


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        if "--status" not in sys.argv and "--set-brightness" not in sys.argv:
            notify("Monitor Layout Failed", str(exc), urgency="critical")
        print(f"Error: {exc}", file=sys.stderr)
        sys.exit(1)
