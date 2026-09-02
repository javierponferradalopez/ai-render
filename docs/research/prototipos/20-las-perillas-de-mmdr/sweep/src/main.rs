//! Barrido de perillas de mmdr 0.3.1, usando la API por etapas del crate.
//!
//! Lee un `configs.json` (lista de configuraciones nombradas, cada una con
//! overrides parciales sobre `LayoutConfig::default()`), los aplica a cada caso
//! `.mmd` y vuelca por combinación el layout en JSON y opcionalmente el SVG/PNG.
//!
//! Las etapas son las que fijó el ticket #8:
//!   parse_mermaid_strict -> compute_layout -> render_svg

use anyhow::{Context, Result};
use mermaid_rs_renderer::config::RenderConfig;
use mermaid_rs_renderer::ir::Direction;
use mermaid_rs_renderer::layout_dump::write_layout_dump;
use mermaid_rs_renderer::render::write_output_png;
use mermaid_rs_renderer::{
    compute_layout, measure_svg_dimensions, parse_mermaid_strict, render_svg, LayoutConfig, Theme,
};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// Mezcla profunda: cada clave de `patch` pisa la de `base`, recursivamente.
fn merge(base: &mut Value, patch: &Value) {
    match (base, patch) {
        (Value::Object(b), Value::Object(p)) => {
            for (k, v) in p {
                merge(b.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (b, p) => *b = p.clone(),
    }
}

fn layout_config_from(overrides: Option<&Value>) -> Result<LayoutConfig> {
    let mut base = serde_json::to_value(LayoutConfig::default())?;
    if let Some(patch) = overrides {
        merge(&mut base, patch);
    }
    serde_json::from_value(base).context("los overrides de layout no encajan en LayoutConfig")
}

fn theme_from(spec: &Value) -> Result<Theme> {
    let name = spec.get("theme").and_then(|v| v.as_str()).unwrap_or("modern");
    let mut theme = match name {
        "modern" => Theme::modern(),
        "mermaid_default" => Theme::mermaid_default(),
        other => Theme::from_name(other)
            .with_context(|| format!("tema desconocido: {other}"))?,
    };
    if let Some(size) = spec.get("font_size").and_then(|v| v.as_f64()) {
        theme.font_size = size as f32;
    }
    Ok(theme)
}

fn direction_from(spec: &Value) -> Option<Direction> {
    match spec.get("direction").and_then(|v| v.as_str())? {
        "TB" | "TD" => Some(Direction::TopDown),
        "LR" => Some(Direction::LeftRight),
        "BT" => Some(Direction::BottomTop),
        "RL" => Some(Direction::RightLeft),
        _ => None,
    }
}

struct Args {
    configs: PathBuf,
    cases: Vec<PathBuf>,
    out: PathBuf,
    svg: bool,
    png: bool,
}

fn parse_args() -> Result<Args> {
    let mut configs = PathBuf::from("configs.json");
    let mut out = PathBuf::from("out");
    let mut cases = Vec::new();
    let (mut svg, mut png) = (false, false);
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--configs" => configs = it.next().context("--configs sin valor")?.into(),
            "--out" => out = it.next().context("--out sin valor")?.into(),
            "--svg" => svg = true,
            "--png" => png = true,
            other => cases.push(PathBuf::from(other)),
        }
    }
    anyhow::ensure!(!cases.is_empty(), "no se han pasado casos .mmd");
    Ok(Args { configs, cases, out, svg, png })
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let specs: Vec<Value> = serde_json::from_str(&fs::read_to_string(&args.configs)?)?;

    for spec in &specs {
        let name = spec
            .get("name")
            .and_then(|v| v.as_str())
            .context("cada config necesita un `name`")?;
        let theme = theme_from(spec)?;
        let layout_cfg = layout_config_from(spec.get("layout"))?;
        let forced = direction_from(spec);
        let dir = args.out.join(name);
        fs::create_dir_all(&dir)?;

        for case in &args.cases {
            let stem = case
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("caso")
                .to_string();
            let source = fs::read_to_string(case)?;

            // Etapa 1: parse estricto. El IR es nuestro a partir de aquí.
            let mut parsed = match parse_mermaid_strict(&source) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{name}\t{stem}\tPARSE_ERROR\t{e}");
                    continue;
                }
            };
            if let Some(d) = forced {
                parsed.graph.direction = d;
            }

            // Etapa 2: layout.
            let layout = compute_layout(&parsed.graph, &theme, &layout_cfg);
            write_layout_dump(&dir.join(format!("{stem}.layout.json")), &layout, &parsed.graph)?;

            // Etapa 3: dibujo.
            if args.svg || args.png {
                let svg = render_svg(&layout, &theme, &layout_cfg);
                if args.svg {
                    fs::write(dir.join(format!("{stem}.svg")), &svg)?;
                }
                if args.png {
                    let d = measure_svg_dimensions(&layout, &layout_cfg, None);
                    let render_cfg = RenderConfig {
                        width: d.width,
                        height: d.height,
                        background: theme.background.clone(),
                    };
                    write_output_png(&svg, &dir.join(format!("{stem}.png")), &render_cfg, &theme)?;
                }
            }
            println!("{name}\t{stem}\t{:.0}x{:.0}", layout.width, layout.height);
        }
    }
    Ok(())
}

