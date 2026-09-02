//! Banco de constructos contra mmdr 0.3.1, por la API por etapas.
//!
//! Corre la tubería que flipchart va a ejecutar —`parse_mermaid_strict` ->
//! vaciado de los siete campos de estilo -> dirección impuesta ->
//! `compute_layout` -> `render_svg`— y por cada caso emite dos cosas:
//!
//!   1. el SVG, en `out/<caso>.svg`
//!   2. una ficha JSON del `Graph` tras el parse, en `out/censo.json`
//!
//! La ficha no es un volcado completo: es lo que hace falta para el careo de
//! `censo.py` —qué ids y etiquetas quedaron en el IR, qué canales laterales
//! vienen llenos— más el veredicto de cada etapa.

use anyhow::{Context, Result};
use mermaid_rs_renderer::ir::{DiagramKind, Direction, Graph};
use mermaid_rs_renderer::parser::parse_mermaid;
use mermaid_rs_renderer::{
    compute_layout, parse_mermaid_strict, render_svg, LayoutConfig, Theme,
};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::PathBuf;

/// Los siete campos que [#11] vacía tras el parse: el estilo aterriza aquí y
/// de aquí no sale. Devuelve los que venían llenos, que es lo que `show` avisa.
fn drain_style(graph: &mut Graph) -> Vec<&'static str> {
    let mut lleno = Vec::new();
    if !graph.class_defs.is_empty() {
        lleno.push("class_defs");
        graph.class_defs.clear();
    }
    if !graph.node_classes.is_empty() {
        lleno.push("node_classes");
        graph.node_classes.clear();
    }
    if !graph.node_styles.is_empty() {
        lleno.push("node_styles");
        graph.node_styles.clear();
    }
    if !graph.subgraph_styles.is_empty() {
        lleno.push("subgraph_styles");
        graph.subgraph_styles.clear();
    }
    if !graph.subgraph_classes.is_empty() {
        lleno.push("subgraph_classes");
        graph.subgraph_classes.clear();
    }
    if !graph.edge_styles.is_empty() {
        lleno.push("edge_styles");
        graph.edge_styles.clear();
    }
    if graph.edge_style_default.is_some() {
        lleno.push("edge_style_default");
        graph.edge_style_default = None;
    }
    lleno
}

fn kind_name(kind: DiagramKind) -> &'static str {
    use DiagramKind::*;
    match kind {
        Flowchart => "Flowchart",
        Class => "Class",
        State => "State",
        Sequence => "Sequence",
        Er => "Er",
        Pie => "Pie",
        Mindmap => "Mindmap",
        Journey => "Journey",
        Timeline => "Timeline",
        Gantt => "Gantt",
        Requirement => "Requirement",
        GitGraph => "GitGraph",
        C4 => "C4",
        Sankey => "Sankey",
        Quadrant => "Quadrant",
        ZenUML => "ZenUML",
        Block => "Block",
        Packet => "Packet",
        Kanban => "Kanban",
        Architecture => "Architecture",
        Radar => "Radar",
        Treemap => "Treemap",
        XYChart => "XYChart",
    }
}

fn dir_name(d: Direction) -> &'static str {
    match d {
        Direction::TopDown => "TB",
        Direction::LeftRight => "LR",
        Direction::BottomTop => "BT",
        Direction::RightLeft => "RL",
    }
}

/// Los canales del `Graph` que no son `nodes`/`edges`/`subgraphs`. Sólo se
/// apuntan los que vienen llenos: la lista completa son cuarenta y pico campos
/// y lo que importa es dónde aterrizó cada constructo.
fn side_channels(g: &Graph) -> Map<String, Value> {
    let mut m = Map::new();
    let mut put = |k: &str, v: Value| {
        m.insert(k.to_string(), v);
    };
    if !g.sequence_participants.is_empty() {
        put("sequence_participants", json!(g.sequence_participants));
    }
    if !g.sequence_frames.is_empty() {
        put("sequence_frames", json!(g.sequence_frames.len()));
    }
    if !g.sequence_notes.is_empty() {
        let labels: Vec<&str> = g.sequence_notes.iter().map(|n| n.label.as_str()).collect();
        put("sequence_notes", json!(labels));
    }
    if !g.sequence_activations.is_empty() {
        put("sequence_activations", json!(g.sequence_activations.len()));
    }
    if !g.sequence_boxes.is_empty() {
        put("sequence_boxes", json!(g.sequence_boxes.len()));
    }
    if !g.state_notes.is_empty() {
        let labels: Vec<&str> = g.state_notes.iter().map(|n| n.label.as_str()).collect();
        put("state_notes", json!(labels));
    }
    if !g.pie_slices.is_empty() {
        put("pie_slices", json!(g.pie_slices.len()));
    }
    if !g.gantt_tasks.is_empty() {
        put("gantt_tasks", json!(g.gantt_tasks.len()));
    }
    if !g.gitgraph.commits.is_empty() {
        put("gitgraph_commits", json!(g.gitgraph.commits.len()));
    }
    if !g.node_links.is_empty() {
        let ids: Vec<&str> = g.node_links.keys().map(|k| k.as_str()).collect();
        put("node_links", json!(ids));
    }
    if !g.mindmap.nodes.is_empty() {
        put("mindmap_nodes", json!(g.mindmap.nodes.len()));
    }
    if !g.timeline.events.is_empty() {
        put("timeline_events", json!(g.timeline.events.len()));
    }
    if !g.xychart.series.is_empty() {
        put("xychart_series", json!(g.xychart.series.len()));
    }
    if !g.quadrant.points.is_empty() {
        put("quadrant_points", json!(g.quadrant.points.len()));
    }
    if g.block.is_some() {
        put("block", json!(true));
    }
    if !g.arch_edge_ports.is_empty() {
        put("arch_edge_ports", json!(g.arch_edge_ports.len()));
    }
    m
}

fn ficha_grafo(g: &Graph) -> Value {
    let nodos: Vec<Value> = g
        .nodes
        .values()
        .map(|n| json!({ "id": n.id, "label": n.label, "shape": format!("{:?}", n.shape) }))
        .collect();
    let aristas: Vec<Value> = g
        .edges
        .iter()
        .map(|e| json!({ "raw": format!("{:?}", e) }))
        .collect();
    let subgrafos: Vec<Value> = g
        .subgraphs
        .iter()
        .map(|s| {
            json!({
                "id": s.id,
                "label": s.label,
                "nodes": s.nodes,
                "direction": s.direction.map(dir_name),
            })
        })
        .collect();
    json!({
        "kind": kind_name(g.kind),
        "direction": dir_name(g.direction),
        "nodes": nodos,
        "edges": aristas,
        "subgraphs": subgrafos,
        "side_channels": Value::Object(side_channels(g)),
    })
}

fn main() -> Result<()> {
    let mut out = PathBuf::from("out");
    let mut casos: Vec<PathBuf> = Vec::new();
    let mut imponer_direccion = true;
    let mut permisivo = false;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--out" => out = it.next().context("--out sin valor")?.into(),
            "--sin-direccion" => imponer_direccion = false,
            // El camino que NO usa flipchart: `parse_mermaid` sin validador.
            "--permisivo" => permisivo = true,
            other => casos.push(PathBuf::from(other)),
        }
    }
    anyhow::ensure!(!casos.is_empty(), "no se han pasado casos .mmd");
    fs::create_dir_all(&out)?;

    let theme = Theme::mermaid_default();
    let cfg = LayoutConfig::default();
    let mut censo: Vec<Value> = Vec::new();

    for caso in &casos {
        let nombre = caso
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("caso")
            .to_string();
        let fuente = fs::read_to_string(caso)?;

        // Etapa 1: parse estricto. Un `Err` aquí es el rechazo que #15 compró.
        let estricto = parse_mermaid_strict(&fuente).map_err(|e| e.to_string());
        let elegido = if permisivo {
            parse_mermaid(&fuente).map_err(|e| e.to_string())
        } else {
            estricto
        };
        let mut parsed = match elegido {
            Ok(p) => p,
            Err(e) => {
                censo.push(json!({
                    "caso": nombre,
                    "parse": "Err",
                    "error": e,
                }));
                continue;
            }
        };

        let estilo = drain_style(&mut parsed.graph);
        let dir_parseada = dir_name(parsed.graph.direction);
        let impuesta = imponer_direccion
            && matches!(parsed.graph.kind, DiagramKind::Flowchart | DiagramKind::Class);
        if impuesta {
            parsed.graph.direction = Direction::LeftRight;
        }
        let ficha = ficha_grafo(&parsed.graph);

        // Etapas 2 y 3. Un pánico aquí es el quinto desenlace de #11; lo
        // atrapamos para que un caso no se lleve el banco entero por delante.
        let render = std::panic::catch_unwind(|| {
            let layout = compute_layout(&parsed.graph, &theme, &cfg);
            render_svg(&layout, &theme, &cfg)
        });

        match render {
            Ok(svg) => {
                fs::write(out.join(format!("{nombre}.svg")), &svg)?;
                censo.push(json!({
                    "caso": nombre,
                    "parse": "Ok",
                    "estilo_vaciado": estilo,
                    "direccion_parseada": dir_parseada,
                    "direccion_impuesta": impuesta,
                    "grafo": ficha,
                    "svg_bytes": svg.len(),
                }));
            }
            Err(_) => censo.push(json!({
                "caso": nombre,
                "parse": "Ok",
                "estilo_vaciado": estilo,
                "grafo": ficha,
                "render": "PANIC",
            })),
        }
    }

    let destino = out.join("censo.json");
    fs::write(&destino, serde_json::to_string_pretty(&censo)?)?;
    eprintln!("{} casos -> {}", censo.len(), destino.display());
    Ok(())
}
