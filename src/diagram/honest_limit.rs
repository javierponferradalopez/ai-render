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

/// A label, a body or a shape of its own. The IR does not tell `Order["Order"]`
/// from a bare `Order` —a label that matches the id is lost—, so the third
/// question is put to the source.
fn declares_itself(node: &Node, body: &Body<'_>) -> bool {
    node.label != node.id || node.shape != NodeShape::Rectangle || body.gives_it_a_body(&node.id)
}

fn in_a_relation(graph: &Graph, id: &str) -> bool {
    graph
        .edges
        .iter()
        .any(|edge| edge.from == id || edge.to == id)
}

/// The source without its first line: the header says what kind of diagram this
/// is and declares no nodes in any Mermaid family.
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

/// Where the line names the `id` —as a whole token, not as a piece of another—
/// and which character comes after it.
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

    fn rules(source: &str) -> Option<String> {
        let parsed = parse_mermaid(source).expect("the test source parses");
        undeclared_nodes(&parsed.graph, source)
    }

    #[test]
    fn a_graph_of_bare_ids_gets_drawn() {
        assert_eq!(rules("flowchart TD\n  A --> B\n"), None);
    }

    #[test]
    fn a_bare_id_next_to_a_labelled_one_is_rejected() {
        let rejection = rules("flowchart TD\n  API[API Layer] --> Db\n").unwrap();

        assert!(rejection.contains("\"Db\"  line 2  — only used in a relation"));
    }

    #[test]
    fn a_label_that_repeats_the_id_is_still_a_label() {
        assert_eq!(rules("flowchart TD\n  API[API Layer] --> Db[Db]\n"), None);
    }

    #[test]
    fn the_leading_case_is_not_a_false_positive() {
        let arch =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cases/arch.mmd"))
                .expect("the leading case is in the repo");

        assert_eq!(rules(&arch), None);
    }

    #[test]
    fn the_node_the_parser_manufactures_from_the_header_is_not_in_the_source() {
        let rejection = rules("flowchart\n  A[One] --> B[Two]\n").unwrap();

        assert!(rejection.contains("\"flowchart\"  — not in your source"));
    }

    #[test]
    fn the_id_the_parser_breaks_off_a_line_it_could_not_read_is_rejected() {
        let rejection =
            rules("flowchart TD\n  One@{ shape: cyl, label: \"X\" }\n  One --> Two[Two]\n")
                .unwrap();

        assert!(rejection.contains("\"One@\"  line 2  — not in your source"));
    }

    #[test]
    fn the_two_causes_travel_together_in_a_single_rejection() {
        let rejection =
            rules("flowchart TD\n  One@{ shape: cyl }\n  Two[Two] --> Three\n").unwrap();

        assert_eq!(
            rejection,
            "2 nodes appear in the drawing that you did not declare.\n  \
             \"One@\"   line 2  — not in your source\n  \
             \"Three\"  line 3  — only used in a relation\n\
             Declare every node you name, and rewrite any line the renderer turned into one."
        );
    }

    #[test]
    fn a_node_declared_bare_and_with_no_relations_is_not_a_phantom() {
        assert_eq!(rules("flowchart TD\n  A[One] --> B[Two]\n  C\n"), None);
    }

    #[test]
    fn the_confrontation_leaves_the_first_line_out() {
        let rejection = rules("flowchart TD\n  TD[One] --> flowchart\n");

        assert!(rejection.unwrap().contains("\"flowchart\""));
    }
}
