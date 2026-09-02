"""Genera el `configs.json` del barrido.

`ejes` mueve una perilla cada vez desde el estado de hoy (el que reproduce
research 08: `Theme::mermaid_default` y `LayoutConfig::default`), para saber cuál
mueve el dibujo y en qué dirección. `combos` cruza las que resultaron mover algo.
"""
import json, sys

HOY = {"name": "hoy", "theme": "mermaid_default"}


def eje(nombre, **spec):
    return {"name": nombre, "theme": "mermaid_default", **spec}


def flow(path, value):
    """Override anidado dentro de layout.flowchart.<path>."""
    d = cur = {}
    keys = path.split(".")
    for k in keys[:-1]:
        cur[k] = {}
        cur = cur[k]
    cur[keys[-1]] = value
    return {"layout": {"flowchart": d}}


def ejes():
    out = [HOY]
    for d in ("LR", "BT", "RL"):
        out.append(eje(f"dir-{d}", direction=d))
    for r in (0.8, 1.0, 1.3, 1.6, 1.78, 2.2, 3.0):
        out.append(eje(f"ar-{r}", layout={"preferred_aspect_ratio": r}))
    for v in (20, 30, 40, 70, 90, 120):
        out.append(eje(f"nodesp-{v}", layout={"node_spacing": v}))
    for v in (20, 30, 40, 70, 90, 120):
        out.append(eje(f"ranksp-{v}", layout={"rank_spacing": v}))
    for t in ("modern", "forest", "neutral", "dark"):
        out.append(eje(f"theme-{t}", theme=t))
    for f in (10, 12, 16, 18):
        out.append(eje(f"font-{f}", font_size=f))
    for v in (10, 20, 40, 60):
        out.append(eje(f"padx-{v}", layout={"node_padding_x": v}))
    for v in (5, 25, 40):
        out.append(eje(f"pady-{v}", layout={"node_padding_y": v}))
    # Las perillas que el ticket no listaba: salen de FlowchartLayoutConfig.
    out.append(eje("obj-off", **flow("objective.enabled", False)))
    for v in (1.2, 1.5, 2.0, 3.0, 4.0, 20.0):
        out.append(eje(f"objar-{v}", **flow("objective.max_aspect_ratio", v)))
    for v in (2, 3, 5, 8, 99):
        out.append(eje(f"wrapmin-{v}", **flow("objective.wrap_min_groups", v)))
    for v in (0.6, 1.6, 2.5):
        out.append(eje(f"wrapmain-{v}", **flow("objective.wrap_main_gap_scale", v)))
    for v in (0.6, 2.0, 3.0):
        out.append(eje(f"wrapcross-{v}", **flow("objective.wrap_cross_gap_scale", v)))
    for v in (0, 2, 12):
        out.append(eje(f"relax-{v}", **flow("objective.edge_relax_passes", v)))
    for v in (0.0, 0.3, 1.5, 3.0):
        out.append(eje(f"backedge-{v}", **flow("objective.backedge_cross_weight", v)))
    out.append(eje("grid-off", **flow("routing.enable_grid_router", False)))
    out.append(eje("snap-off", **flow("routing.snap_ports_to_grid", False)))
    for v in (4, 8, 32, 64):
        out.append(eje(f"cell-{v}", **flow("routing.grid_cell", v)))
    for v in (0.0, 0.2, 1.5, 4.0):
        out.append(eje(f"turn-{v}", **flow("routing.turn_penalty", v)))
    for v in (0.0, 0.4, 3.0, 8.0):
        out.append(eje(f"occ-{v}", **flow("routing.occupancy_weight", v)))
    for v in (1, 2, 8, 16, 32):
        out.append(eje(f"passes-{v}", **flow("order_passes", v)))
    for v in (-1.0, -0.5, 0.5, 1.0):
        out.append(eje(f"bias-{v}", **flow("port_side_bias", v)))
    for v in (0.0, 0.4, 0.8):
        out.append(eje(f"portpad-{v}", **flow("port_pad_ratio", v)))
    out.append(eje("autosp-off", **flow("auto_spacing.enabled", False)))
    return out


if __name__ == "__main__":
    which = sys.argv[1] if len(sys.argv) > 1 else "ejes"
    print(json.dumps({"ejes": ejes}[which](), indent=1))
