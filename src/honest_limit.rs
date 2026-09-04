use mermaid_rs_renderer::{Graph, Node, NodeShape};

pub fn undeclared_nodes(graph: &Graph, source: &str) -> Option<String> {
    let body = Body::of(source);
    let anyone_declares_itself = graph
        .nodes
        .values()
        .any(|node| declares_itself(node, &body));

    let mut found: Vec<Undeclared> = graph
        .nodes
        .values()
        .filter_map(|node| judge(node, graph, &body, anyone_declares_itself))
        .collect();
    if found.is_empty() {
        return None;
    }
    found.sort_by(|one, other| {
        (one.line.unwrap_or(usize::MAX), &one.id)
            .cmp(&(other.line.unwrap_or(usize::MAX), &other.id))
    });
    Some(worded(&found))
}

#[derive(Debug)]
enum Cause {
    NotInSource,
    OnlyInARelation,
}

#[derive(Debug)]
struct Undeclared {
    id: String,
    line: Option<usize>,
    cause: Cause,
}

fn judge(
    node: &Node,
    graph: &Graph,
    body: &Body<'_>,
    anyone_declares_itself: bool,
) -> Option<Undeclared> {
    let Some(line) = body.line_naming(&node.id) else {
        return Some(Undeclared {
            id: node.id.clone(),
            line: body.line_holding(&node.id),
            cause: Cause::NotInSource,
        });
    };
    let ghost =
        anyone_declares_itself && !declares_itself(node, body) && in_a_relation(graph, &node.id);
    ghost.then(|| Undeclared {
        id: node.id.clone(),
        line: Some(line),
        cause: Cause::OnlyInARelation,
    })
}

/// Etiqueta, cuerpo o forma propia. El IR no distingue `Order["Order"]` de
/// `Order` a secas —la etiqueta que coincide con el id se pierde—, así que la
/// tercera pregunta se la hacemos al fuente.
fn declares_itself(node: &Node, body: &Body<'_>) -> bool {
    node.label != node.id || node.shape != NodeShape::Rectangle || body.gives_it_a_body(&node.id)
}

fn in_a_relation(graph: &Graph, id: &str) -> bool {
    graph
        .edges
        .iter()
        .any(|edge| edge.from == id || edge.to == id)
}

/// El fuente sin su primera línea: la cabecera dice de qué tipo es el diagrama y
/// no declara nodos en ninguna familia de Mermaid.
#[derive(Debug)]
struct Body<'a> {
    lines: Vec<(usize, &'a str)>,
}

impl<'a> Body<'a> {
    fn of(source: &'a str) -> Self {
        Self {
            lines: source
                .lines()
                .enumerate()
                .skip(1)
                .map(|(index, line)| (index + 1, line))
                .collect(),
        }
    }

    fn line_naming(&self, id: &str) -> Option<usize> {
        self.lines
            .iter()
            .find(|(_, line)| named_in(line, id).next().is_some())
            .map(|(number, _)| *number)
    }

    fn line_holding(&self, id: &str) -> Option<usize> {
        self.lines
            .iter()
            .find(|(_, line)| line.contains(id))
            .map(|(number, _)| *number)
    }

    fn gives_it_a_body(&self, id: &str) -> bool {
        self.lines.iter().any(|(_, line)| {
            named_in(line, id).any(|next| matches!(next, Some('[' | '(' | '{' | '>')))
        })
    }
}

/// Dónde nombra la línea al `id` —como token entero, no como trozo de otro— y
/// qué carácter viene detrás.
fn named_in<'a>(line: &'a str, id: &'a str) -> impl Iterator<Item = Option<char>> + 'a {
    let a_token = !id.is_empty() && id.chars().all(part_of_an_id);
    a_token
        .then(|| line.match_indices(id))
        .into_iter()
        .flatten()
        .filter_map(move |(at, _)| {
            let before = line[..at].chars().next_back();
            let after = line[at + id.len()..].chars().next();
            let whole = !before.is_some_and(part_of_an_id) && !after.is_some_and(part_of_an_id);
            whole.then_some(after)
        })
}

fn part_of_an_id(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn worded(found: &[Undeclared]) -> String {
    let names: Vec<String> = found.iter().map(|one| format!("\"{}\"", one.id)).collect();
    let places: Vec<String> = found
        .iter()
        .map(|one| {
            one.line
                .map(|line| format!("line {line}"))
                .unwrap_or_default()
        })
        .collect();
    let name_width = names.iter().map(String::len).max().unwrap_or_default();
    let place_width = places.iter().map(String::len).max().unwrap_or_default();

    let mut text = if found.len() == 1 {
        "1 node appears in the drawing that you did not declare.".to_string()
    } else {
        format!(
            "{} nodes appear in the drawing that you did not declare.",
            found.len()
        )
    };
    for ((one, name), place) in found.iter().zip(&names).zip(&places) {
        let cause = match one.cause {
            Cause::NotInSource => "not in your source",
            Cause::OnlyInARelation => "only used in a relation",
        };
        let column = if place_width == 0 {
            String::new()
        } else {
            format!("{place:place_width$}  ")
        };
        text.push_str(&format!("\n  {name:name_width$}  {column}— {cause}"));
    }
    text.push_str(
        "\nDeclare every node you name, and rewrite any line the renderer turned into one.",
    );
    text
}

#[cfg(test)]
mod tests {
    use mermaid_rs_renderer::parse_mermaid;

    use super::*;

    fn reglas(source: &str) -> Option<String> {
        let parsed = parse_mermaid(source).expect("el fuente de la prueba parsea");
        undeclared_nodes(&parsed.graph, source)
    }

    #[test]
    fn un_grafo_de_ids_desnudos_se_dibuja() {
        assert_eq!(reglas("flowchart TD\n  A --> B\n"), None);
    }

    #[test]
    fn un_id_desnudo_al_lado_de_uno_con_etiqueta_se_rechaza() {
        let rechazo = reglas("flowchart TD\n  API[API Layer] --> Db\n").unwrap();

        assert!(rechazo.contains("\"Db\"  line 2  — only used in a relation"));
    }

    #[test]
    fn una_etiqueta_que_repite_el_id_sigue_siendo_etiqueta() {
        assert_eq!(reglas("flowchart TD\n  API[API Layer] --> Db[Db]\n"), None);
    }

    #[test]
    fn el_caso_protagonista_no_es_un_falso_positivo() {
        let arch = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/docs/research/prototipos/13-mmdr-frente-a-termaid/arch.mmd"
        ))
        .expect("el caso protagonista está en el repo");

        assert_eq!(reglas(&arch), None);
    }

    #[test]
    fn el_nodo_que_el_parser_fabrica_de_la_cabecera_no_esta_en_el_fuente() {
        let rechazo = reglas("flowchart\n  A[Uno] --> B[Dos]\n").unwrap();

        assert!(rechazo.contains("\"flowchart\"  — not in your source"));
    }

    #[test]
    fn el_id_que_el_parser_parte_de_una_linea_que_no_supo_leer_se_rechaza() {
        let rechazo =
            reglas("flowchart TD\n  Uno@{ shape: cyl, label: \"X\" }\n  Uno --> Dos[Dos]\n")
                .unwrap();

        assert!(rechazo.contains("\"Uno@\"  line 2  — not in your source"));
    }

    #[test]
    fn las_dos_causas_van_juntas_en_un_solo_rechazo() {
        let rechazo = reglas("flowchart TD\n  Uno@{ shape: cyl }\n  Dos[Dos] --> Tres\n").unwrap();

        assert_eq!(
            rechazo,
            "2 nodes appear in the drawing that you did not declare.\n  \
             \"Uno@\"  line 2  — not in your source\n  \
             \"Tres\"  line 3  — only used in a relation\n\
             Declare every node you name, and rewrite any line the renderer turned into one."
        );
    }

    #[test]
    fn un_nodo_declarado_a_secas_y_sin_relaciones_no_es_un_fantasma() {
        assert_eq!(reglas("flowchart TD\n  A[Uno] --> B[Dos]\n  C\n"), None);
    }

    #[test]
    fn el_careo_deja_fuera_la_primera_linea() {
        let rechazo = reglas("flowchart TD\n  TD[Uno] --> flowchart\n");

        assert!(rechazo.unwrap().contains("\"flowchart\""));
    }
}
