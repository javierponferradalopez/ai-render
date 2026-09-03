use mermaid_rs_renderer::{LayoutConfig, Theme, compute_layout, parse_mermaid, render_svg};

#[derive(Debug)]
pub struct Drawing {
    pub svg: String,
    pub nodes: usize,
    pub edges: usize,
}

/// Los de fábrica, sin tocar: ninguna perilla de mmdr mejora el dibujo y varias
/// hacen daño. Medido en DECISIONS §3.4.
fn theme() -> Theme {
    Theme::mermaid_default()
}

pub fn draw(source: &str) -> Result<Drawing, String> {
    let parsed = parse_mermaid(source).map_err(|error| error.to_string())?;
    let (theme, config) = (theme(), LayoutConfig::default());

    let positioned_scene = compute_layout(&parsed.graph, &theme, &config);

    Ok(Drawing {
        svg: render_svg(&positioned_scene, &theme, &config),
        nodes: parsed.graph.nodes.len(),
        edges: parsed.graph.edges.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOS_NODOS: &str = "flowchart TD\n  A[Uno] --> B[Dos]\n";

    /// El caso protagonista del prototipo 13/20: cuatro grupos y siete aristas.
    fn arch() -> String {
        std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/docs/research/prototipos/13-mmdr-frente-a-termaid/arch.mmd"
        ))
        .expect("el caso protagonista está en el repo")
    }

    #[test]
    fn la_tuberia_devuelve_un_svg_con_las_etiquetas_dentro() {
        let dibujo = draw(DOS_NODOS).unwrap();

        assert!(dibujo.svg.starts_with("<svg"));
        assert!(dibujo.svg.contains("Uno"));
        assert!(dibujo.svg.contains("Dos"));
    }

    #[test]
    fn la_tuberia_cuenta_lo_que_ha_dibujado() {
        let dibujo = draw(DOS_NODOS).unwrap();

        assert_eq!((dibujo.nodes, dibujo.edges), (2, 1));
    }

    #[test]
    fn el_tema_es_el_de_mermaid_y_no_el_moderno_de_14_px() {
        assert_eq!(theme().font_size, Theme::mermaid_default().font_size);
        assert_ne!(theme().font_size, Theme::modern().font_size);
    }

    #[test]
    fn lo_que_no_es_mermaid_no_llega_a_dibujarse() {
        assert!(draw("esto no es Mermaid").is_err());
    }

    #[test]
    fn el_caso_protagonista_sale_entero_por_la_tuberia() {
        let dibujo = draw(&arch()).unwrap();

        assert_eq!((dibujo.nodes, dibujo.edges), (8, 7));
        for grupo in ["API", "Application", "Domain", "Infrastructure"] {
            assert!(dibujo.svg.contains(grupo), "falta el grupo {grupo}");
        }
    }
}
