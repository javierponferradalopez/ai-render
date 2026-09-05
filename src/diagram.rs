use std::panic::{AssertUnwindSafe, catch_unwind};

use mermaid_rs_renderer::{
    LayoutConfig, ParseError, ParseOutput, Theme, compute_layout, parse_mermaid,
    parse_mermaid_strict, render_svg,
};

use crate::honest_limit::undeclared_nodes;
use crate::house_style;

#[derive(Debug)]
pub struct Drawing {
    pub svg: String,
    pub nodes: usize,
    pub edges: usize,
    pub notes: Vec<&'static str>,
}

impl Drawing {
    /// La única realimentación que el agente tiene sobre el dibujo, porque la
    /// imagen no vuelve nunca al contexto.
    pub fn recount(&self) -> String {
        format!(
            "{}, {}",
            plural(self.nodes, "node"),
            plural(self.edges, "edge")
        )
    }

    /// Los avisos van detrás, uno por línea: el agente lee primero el desenlace
    /// y después el precio.
    pub fn noted_after(&self, text: String) -> String {
        self.notes.iter().fold(text, |mut told, note| {
            told.push('\n');
            told.push_str(note);
            told
        })
    }
}

fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("{count} {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

/// Los cuatro desenlaces de `show` que no dibujan. El de pánico es el único que
/// dice que la culpa es nuestra, a propósito: si le pedimos al agente que
/// arregle su diagrama, lo intentará en bucle sobre algo que no tiene arreglo.
#[derive(Debug)]
pub enum Rejection {
    InvalidInput(String),
    Unparsed(String),
    Undeclared(String),
    RendererPanicked,
}

impl Rejection {
    pub fn outcome(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "invalid input",
            Self::Unparsed(_) => "parse error",
            Self::Undeclared(_) => "undeclared nodes",
            Self::RendererPanicked => "renderer panic",
        }
    }

    pub fn told_about(&self, view_id: &str) -> String {
        match self {
            Self::InvalidInput(diagnostic)
            | Self::Unparsed(diagnostic)
            | Self::Undeclared(diagnostic) => {
                format!(
                    "Rejected: nothing was drawn; view \"{view_id}\" is unchanged.\n{diagnostic}"
                )
            }
            Self::RendererPanicked => format!(
                "Rejected: the renderer failed on this diagram; nothing was drawn.\n\
                 View \"{view_id}\" is unchanged. This is a bug in the flipchart, not in your \
                 diagram — try a simpler diagram, or the same one with fewer nodes."
            ),
        }
    }
}

/// Los de fábrica, sin tocar: ninguna perilla de mmdr mejora el dibujo y varias
/// hacen daño. Medido en DECISIONS §3.4.
fn theme() -> Theme {
    Theme::mermaid_default()
}

pub fn draw(source: &str) -> Result<Drawing, Rejection> {
    let mut parsed = guarded(|| read(source))?.map_err(Rejection::Unparsed)?;

    if let Some(diagnostic) = undeclared_nodes(&parsed.graph, source) {
        return Err(Rejection::Undeclared(diagnostic));
    }

    let notes = house_style::imposed_on(&mut parsed, source);
    let graph = parsed.graph;

    let (theme, config) = (theme(), LayoutConfig::default());
    let svg = guarded(|| {
        let positioned_scene = compute_layout(&graph, &theme, &config);
        render_svg(&positioned_scene, &theme, &config)
    })?;

    Ok(Drawing {
        svg,
        nodes: graph.nodes.len(),
        edges: graph.edges.len(),
        notes,
    })
}

/// Servidor y Visor comparten proceso, así que un pánico sin capturar se
/// llevaría la pizarra entera por delante, callando.
fn guarded<T>(step: impl FnOnce() -> T) -> Result<T, Rejection> {
    catch_unwind(AssertUnwindSafe(step)).map_err(|_| Rejection::RendererPanicked)
}

/// Se entra por el camino permisivo; el strict sólo se paga cuando ya no se va a
/// dibujar nada, y sólo para obtener el `ParseError` tipado del mensaje.
fn read(source: &str) -> Result<ParseOutput, String> {
    match parse_mermaid(source) {
        Ok(parsed) => Ok(parsed),
        Err(permissive) => Err(match parse_mermaid_strict(source) {
            Err(typed) => diagnosed(&typed),
            Ok(_) => permissive.to_string(),
        }),
    }
}

fn diagnosed(error: &ParseError) -> String {
    match error {
        ParseError::UnknownParticipant {
            name,
            line,
            candidates,
        } if candidates.is_empty() => {
            format!("Unknown node \"{name}\" at line {line} — it was never declared.")
        }
        ParseError::UnknownParticipant {
            name,
            line,
            candidates,
        } => format!(
            "Unknown node \"{name}\" at line {line} — did you mean {}?",
            shortlist(
                candidates
                    .iter()
                    .map(|candidate| format!("\"{candidate}\""))
            )
        ),
        ParseError::UnclosedSubgraph { opened_at } => {
            format!("A subgraph opened at line {opened_at} was never closed.")
        }
        ParseError::UnexpectedToken {
            line,
            col,
            found,
            expected,
        } => format!(
            "Unexpected token at line {line}, column {col} — found \"{found}\", expected {}.",
            shortlist(expected.split(',').map(|one| one.trim().to_string()))
        ),
        unclassified => {
            format!("The flipchart did not classify this failure: {unclassified}")
        }
    }
}

fn shortlist(items: impl Iterator<Item = String>) -> String {
    let first_three: Vec<String> = items.take(3).collect();
    match first_three.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOS_NODOS: &str = "flowchart TD\n  A[Uno] --> B[Dos]\n";

    /// El **caso protagonista**, que es entender un refactor antes de hacerlo:
    /// cuatro grupos y siete aristas.
    fn arch() -> String {
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/casos/arch.mmd"))
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

    #[test]
    fn lo_que_no_parsea_es_el_desenlace_del_parse_error() {
        let rechazo = draw("esto no es Mermaid").unwrap_err();

        assert_eq!(rechazo.outcome(), "parse error");
    }

    #[test]
    fn un_nodo_que_el_agente_no_declaro_es_el_desenlace_de_las_reglas() {
        let rechazo = draw("flowchart TD\n  API[API Layer] --> Db\n").unwrap_err();

        assert_eq!(rechazo.outcome(), "undeclared nodes");
    }

    #[test]
    fn el_parse_error_va_antes_que_las_reglas_porque_sin_grafo_no_hay_reglas() {
        let rechazo = draw("no es Mermaid\n  API[API Layer] --> Db\n").unwrap_err();

        assert_eq!(rechazo.outcome(), "parse error");
    }

    #[test]
    fn todo_rechazo_que_no_sea_el_panico_abre_con_la_linea_fija() {
        let rechazo = draw("esto no es Mermaid").unwrap_err();

        assert!(
            rechazo
                .told_about("propuesto")
                .starts_with("Rejected: nothing was drawn; view \"propuesto\" is unchanged.\n")
        );
    }

    #[test]
    fn el_panico_es_el_unico_rechazo_que_dice_que_la_culpa_es_nuestra() {
        let texto = Rejection::RendererPanicked.told_about("propuesto");

        assert_eq!(
            texto,
            "Rejected: the renderer failed on this diagram; nothing was drawn.\n\
             View \"propuesto\" is unchanged. This is a bug in the flipchart, not in your \
             diagram — try a simpler diagram, or the same one with fewer nodes."
        );
    }

    #[test]
    fn el_token_inesperado_lleva_linea_columna_y_lo_encontrado() {
        let texto = diagnosed(&ParseError::UnexpectedToken {
            line: 4,
            col: 3,
            found: "-->".to_string(),
            expected: "class".to_string(),
        });

        assert_eq!(
            texto,
            "Unexpected token at line 4, column 3 — found \"-->\", expected class."
        );
    }

    #[test]
    fn un_expected_largo_se_queda_en_los_tres_primeros() {
        let texto = diagnosed(&ParseError::UnexpectedToken {
            line: 4,
            col: 3,
            found: "-->".to_string(),
            expected: "class, }, an identifier, a comment".to_string(),
        });

        assert!(texto.ends_with("expected class, } or an identifier."));
    }

    #[test]
    fn el_participante_desconocido_ofrece_los_candidatos() {
        let texto = diagnosed(&ParseError::UnknownParticipant {
            name: "Ordr".to_string(),
            line: 6,
            candidates: vec!["Order".to_string()],
        });

        assert_eq!(
            texto,
            "Unknown node \"Ordr\" at line 6 — did you mean \"Order\"?"
        );
    }

    #[test]
    fn el_participante_desconocido_sin_candidatos_no_inventa_ninguno() {
        let texto = diagnosed(&ParseError::UnknownParticipant {
            name: "Ordr".to_string(),
            line: 6,
            candidates: Vec::new(),
        });

        assert_eq!(
            texto,
            "Unknown node \"Ordr\" at line 6 — it was never declared."
        );
    }

    #[test]
    fn el_subgrafo_sin_cerrar_dice_donde_se_abrio() {
        let texto = diagnosed(&ParseError::UnclosedSubgraph { opened_at: 3 });

        assert_eq!(texto, "A subgraph opened at line 3 was never closed.");
    }

    #[test]
    fn la_variante_sin_clasificar_admite_que_no_la_hemos_clasificado() {
        let texto = diagnosed(&ParseError::InvalidDirective {
            line: 2,
            col: 1,
            directive: "init".to_string(),
            reason: "empty body".to_string(),
        });

        assert!(texto.starts_with("The flipchart did not classify this failure: "));
    }
}
