use mermaid_rs_renderer::{DiagramKind, Direction, Graph, ParseOutput};

/// Uno solo cubre las tres formas de pedir píxeles —estilo, `click` e
/// `%%{init}%%`—: al agente no le hace falta el inventario, le hace falta saber
/// que aquí decidimos nosotros cómo se ve.
const STYLE_DROPPED: &str = "Note: style directives (classDef, class, style, linkStyle) \
     and click links were dropped — the flipchart decides how views look. The view was drawn.";

/// Éste dice algo distinto del de estilo: aquél decide, éste admite que no
/// sabemos dibujarlo.
const NAMESPACE_NOT_DRAWN: &str =
    "Note: the flipchart could not draw namespace here; the classes were drawn without it.";

const NOTES_NOT_DRAWN: &str =
    "Note: the flipchart could not draw notes here; the classes were drawn without them.";

const DIRECTION_IMPOSED: &str = "Note: the flipchart lays diagrams out left to right; the direction in your source \
     was ignored. The view was drawn.";

/// Vacía el estilo del IR e impone la dirección, y devuelve los avisos **por lo
/// que venía, no por lo que tuvo efecto**: un `classDef` que ninguna clase usaba
/// también avisa, porque el agente creyó que estaba pintando. Nunca rechaza — los
/// tres acompañan a una Vista que se dibuja.
pub fn imposed_on(parsed: &mut ParseOutput, source: &str) -> Vec<&'static str> {
    let mut notes = Vec::new();

    if style_came_in(parsed) {
        notes.push(STYLE_DROPPED);
    }
    empty_the_style(parsed);

    if parsed.graph.kind == DiagramKind::Class {
        notes.extend(structure_mmdr_cannot_draw(source));
    }
    if lays_out_left_to_right(parsed.graph.kind) {
        impose_left_to_right(&mut parsed.graph);
        if source_declared_another_direction(source) {
            notes.push(DIRECTION_IMPOSED);
        }
    }

    notes
}

/// Los nueve canales del §3.2. Se miran en el IR y no en el texto: por muchas
/// formas nuevas de escribir estilo que Mermaid invente, para tener efecto tienen
/// que aterrizar aquí.
fn style_came_in(parsed: &ParseOutput) -> bool {
    let graph = &parsed.graph;
    !graph.class_defs.is_empty()
        || !graph.node_classes.is_empty()
        || !graph.node_styles.is_empty()
        || !graph.subgraph_styles.is_empty()
        || !graph.subgraph_classes.is_empty()
        || !graph.edge_styles.is_empty()
        || graph.edge_style_default.is_some()
        || !graph.node_links.is_empty()
        || parsed.init_config.is_some()
}

fn empty_the_style(parsed: &mut ParseOutput) {
    let graph = &mut parsed.graph;
    graph.class_defs.clear();
    graph.node_classes.clear();
    graph.node_styles.clear();
    graph.subgraph_styles.clear();
    graph.subgraph_classes.clear();
    graph.edge_styles.clear();
    graph.edge_style_default = None;
    graph.node_links.clear();
    parsed.init_config = None;
}

fn lays_out_left_to_right(kind: DiagramKind) -> bool {
    matches!(kind, DiagramKind::Flowchart | DiagramKind::Class)
}

/// La imposición baja a los grupos: un `subgraph` con `direction` propia es la
/// misma perilla en manos del agente, dentro del grupo.
fn impose_left_to_right(graph: &mut Graph) {
    graph.direction = Direction::LeftRight;
    for subgraph in &mut graph.subgraphs {
        subgraph.direction = None;
    }
}

/// Dos deudas de mmdr con nombre, no una política: `namespace` es cómo un
/// diagrama de clases dice *módulo*, y perderlo en silencio sería que el agente
/// hable de una caja que no está en pantalla.
fn structure_mmdr_cannot_draw(source: &str) -> Vec<&'static str> {
    let mut notes = Vec::new();
    if opens_a_line(source, "namespace") {
        notes.push(NAMESPACE_NOT_DRAWN);
    }
    if opens_a_line(source, "note") {
        notes.push(NOTES_NOT_DRAWN);
    }
    notes
}

/// mmdr **no distingue `flowchart TB` de `flowchart`** —el parser inicializa a
/// `TopDown` y luego pisa—, así que la pregunta se le hace al fuente.
fn source_declared_another_direction(source: &str) -> bool {
    written_directions(source).any(|direction| direction != "LR")
}

fn written_directions(source: &str) -> impl Iterator<Item = &str> {
    speakable_lines(source).filter_map(|line| {
        let mut words = line.split_whitespace();
        let opening = words.next()?;
        let named = matches!(opening, "flowchart" | "graph" | "direction");
        named
            .then(|| words.next())
            .flatten()
            .map(|word| word.trim_end_matches(';'))
            .filter(|word| matches!(*word, "TB" | "TD" | "BT" | "RL" | "LR"))
    })
}

fn opens_a_line(source: &str, word: &str) -> bool {
    speakable_lines(source).any(|line| line.split_whitespace().next() == Some(word))
}

fn speakable_lines(source: &str) -> impl Iterator<Item = &str> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with("%%"))
}

#[cfg(test)]
mod tests {
    use mermaid_rs_renderer::parse_mermaid;

    use super::*;

    fn imposed(source: &str) -> (ParseOutput, Vec<&'static str>) {
        let mut parsed = parse_mermaid(source).expect("el fuente de la prueba parsea");
        let notes = imposed_on(&mut parsed, source);
        (parsed, notes)
    }

    fn notes(source: &str) -> Vec<&'static str> {
        imposed(source).1
    }

    fn graph(source: &str) -> Graph {
        imposed(source).0.graph
    }

    #[test]
    fn el_classdef_se_vacia_del_ir() {
        let dibujo =
            graph("flowchart TD\n  classDef danger fill:#f00\n  A[Uno]:::danger --> B[Dos]\n");

        assert!(dibujo.class_defs.is_empty());
        assert!(dibujo.node_classes.is_empty());
    }

    #[test]
    fn el_style_de_un_nodo_se_vacia_del_ir() {
        let dibujo = graph("flowchart TD\n  A[Uno] --> B[Dos]\n  style A fill:#f00\n");

        assert!(dibujo.node_styles.is_empty());
    }

    #[test]
    fn el_linkstyle_se_vacia_del_ir() {
        let dibujo = graph("flowchart TD\n  A[Uno] --> B[Dos]\n  linkStyle 0 stroke:#f00\n");

        assert!(dibujo.edge_styles.is_empty() && dibujo.edge_style_default.is_none());
    }

    #[test]
    fn el_click_se_vacia_del_ir() {
        let dibujo =
            graph("flowchart TD\n  A[Uno] --> B[Dos]\n  click A \"https://ejemplo\" \"Ir\"\n");

        assert!(dibujo.node_links.is_empty());
    }

    #[test]
    fn el_init_se_vacia_del_parse_output() {
        let (parsed, _) =
            imposed("%%{init: {'theme':'dark'}}%%\nflowchart TD\n  A[Uno] --> B[Dos]\n");

        assert!(parsed.init_config.is_none());
    }

    #[test]
    fn el_estilo_descartado_lleva_un_solo_aviso_para_todo() {
        let avisos = notes(
            "%%{init: {'theme':'dark'}}%%\nflowchart LR\n  classDef danger fill:#f00\n  \
             A[Uno]:::danger --> B[Dos]\n  style B fill:#0f0\n  \
             click A \"https://ejemplo\" \"Ir\"\n",
        );

        assert_eq!(avisos, vec![STYLE_DROPPED]);
    }

    #[test]
    fn un_classdef_que_nadie_usa_avisa_igual_porque_el_agente_creyo_que_pintaba() {
        let avisos = notes("flowchart LR\n  classDef danger fill:#f00\n  A[Uno] --> B[Dos]\n");

        assert_eq!(avisos, vec![STYLE_DROPPED]);
    }

    #[test]
    fn un_diagrama_sin_estilo_no_paga_el_aviso() {
        assert!(notes("flowchart LR\n  A[Uno] --> B[Dos]\n").is_empty());
    }

    #[test]
    fn la_direccion_se_impone_en_el_flowchart() {
        assert_eq!(
            graph("flowchart TB\n  A[Uno] --> B[Dos]\n").direction,
            Direction::LeftRight
        );
    }

    #[test]
    fn la_direccion_se_impone_en_el_diagrama_de_clases() {
        assert_eq!(
            graph("classDiagram\n  direction TB\n  Pedido <|-- Venta\n").direction,
            Direction::LeftRight
        );
    }

    #[test]
    fn la_imposicion_baja_a_la_direccion_de_cada_grupo() {
        let dibujo =
            graph("flowchart TD\n  subgraph API\n    direction RL\n    A[Uno] --> B[Dos]\n  end\n");

        assert!(
            dibujo
                .subgraphs
                .iter()
                .all(|grupo| grupo.direction.is_none())
        );
    }

    #[test]
    fn fuera_de_las_dos_familias_no_se_toca_la_direccion() {
        let dibujo = graph("stateDiagram-v2\n  direction TB\n  [*] --> Activo\n");

        assert_eq!(dibujo.direction, Direction::TopDown);
    }

    #[test]
    fn una_direccion_declarada_distinta_se_avisa() {
        assert_eq!(
            notes("flowchart TB\n  A[Uno] --> B[Dos]\n"),
            vec![DIRECTION_IMPOSED]
        );
    }

    #[test]
    fn una_cabecera_desnuda_no_declaraba_direccion_y_no_se_paga() {
        assert!(notes("flowchart\n  A[Uno] --> B[Dos]\n").is_empty());
    }

    #[test]
    fn una_cabecera_que_ya_pedia_lr_no_se_avisa() {
        assert!(notes("flowchart LR\n  A[Uno] --> B[Dos]\n").is_empty());
    }

    #[test]
    fn la_direccion_de_un_grupo_tambien_es_una_direccion_declarada() {
        let avisos =
            notes("flowchart LR\n  subgraph API\n    direction TB\n    A[Uno] --> B[Dos]\n  end\n");

        assert_eq!(avisos, vec![DIRECTION_IMPOSED]);
    }

    #[test]
    fn fuera_de_las_dos_familias_una_direccion_declarada_no_avisa() {
        assert!(notes("stateDiagram-v2\n  direction TB\n  [*] --> Activo\n").is_empty());
    }

    #[test]
    fn el_namespace_que_mmdr_no_dibuja_se_avisa() {
        let avisos = notes("classDiagram\n  namespace Dominio {\n    class Pedido\n  }\n");

        assert_eq!(avisos, vec![NAMESPACE_NOT_DRAWN]);
    }

    #[test]
    fn la_nota_que_mmdr_no_dibuja_se_avisa() {
        let avisos = notes("classDiagram\n  note for Pedido \"la nota\"\n  class Pedido\n");

        assert_eq!(avisos, vec![NOTES_NOT_DRAWN]);
    }

    #[test]
    fn el_namespace_fuera_del_diagrama_de_clases_no_se_avisa() {
        assert!(notes("flowchart LR\n  namespace --> B[Dos]\n").is_empty());
    }

    #[test]
    fn los_avisos_se_acumulan() {
        let avisos = notes(
            "%%{init: {'theme':'dark'}}%%\nclassDiagram\n  direction TB\n  \
             namespace Dominio {\n    class Pedido\n  }\n  note \"la nota\"\n",
        );

        assert_eq!(
            avisos,
            vec![
                STYLE_DROPPED,
                NAMESPACE_NOT_DRAWN,
                NOTES_NOT_DRAWN,
                DIRECTION_IMPOSED
            ]
        );
    }

    #[test]
    fn un_comentario_no_declara_ni_direccion_ni_estructura() {
        assert!(
            notes("classDiagram\n  %% direction TB\n  %% namespace Dominio\n  class Pedido\n")
                .is_empty()
        );
    }
}
