use mermaid_rs_renderer::{DiagramKind, Direction, Graph, ParseOutput};

/// A single one covers the three ways of asking for pixels —style, `click` and
/// `%%{init}%%`—: the agent does not need the inventory, it needs to know that
/// here it is we who decide how things look.
const STYLE_DROPPED: &str = "Note: style directives (classDef, class, style, linkStyle) \
     and click links were dropped — the flipchart decides how views look. The view was drawn.";

/// This one says something different from the style one: that one decides, this
/// one admits we do not know how to draw it.
const NAMESPACE_NOT_DRAWN: &str =
    "Note: the flipchart could not draw namespace here; the classes were drawn without it.";

const NOTES_NOT_DRAWN: &str =
    "Note: the flipchart could not draw notes here; the classes were drawn without them.";

const DIRECTION_IMPOSED: &str = "Note: the flipchart lays diagrams out left to right; the direction in your source \
     was ignored. The view was drawn.";

/// The fourth says the same as the `namespace` one —we do not know how to draw
/// it— about what travels **inside** the label's text, which is where the
/// emptying of the nine channels does not reach. And it names the one construct
/// that does come out right, because it is the one the agent uses in fifteen out
/// of seventeen diagrams and taking it away would be charging it for something
/// that works.
const MARKUP_DRAWN_AS_TEXT: &str = "Note: only <br> is rendered inside labels; other tags, HTML entities and #-escapes \
     reached the drawing as literal text. The view was drawn — write those labels as plain text.";

/// Empties the style out of the IR and imposes the direction, and returns the
/// notes **for what came in, not for what took effect**: a `classDef` no class
/// was using still warns, because the agent believed it was painting. It never
/// rejects — all four accompany a View that gets drawn.
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
    if any_label_carries_markup(&parsed.graph) {
        notes.push(MARKUP_DRAWN_AS_TEXT);
    }

    notes
}

/// The nine channels of ADR-0006. They are looked at in the IR and not in the
/// text: however many new ways of writing style Mermaid invents, to take effect
/// they have to land here.
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

/// The imposition reaches down into the groups: a `subgraph` with a `direction`
/// of its own is the same knob in the agent's hands, inside the group.
fn impose_left_to_right(graph: &mut Graph) {
    graph.direction = Direction::LeftRight;
    for subgraph in &mut graph.subgraphs {
        subgraph.direction = None;
    }
}

/// Two named mmdr debts, not a policy: `namespace` is how a class diagram says
/// *module*, and losing it silently would mean the agent talking about a box
/// that is not on screen.
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

/// The only construct inside a label that mmdr interprets is `<br>` —like that,
/// or `<br/>`—. Everything else shaped like markup lands in the drawing exactly
/// as it was written, and it is visible garbage that **nobody else can fix**:
/// the agent is blind and the user does not read the source.
///
/// The question is put to the **IR's labels** and not to the source, because
/// that is where no Mermaid syntax is left to mistake for markup: the `&` in
/// `A & B` is not an entity, and `-->` is not a tag.
fn any_label_carries_markup(graph: &Graph) -> bool {
    labels(graph).any(reads_as_markup)
}

/// The text that reaches the drawing in the two families we promise: that of
/// the nodes, that of the groups and that of the edges.
fn labels(graph: &Graph) -> impl Iterator<Item = &str> {
    let nodes = graph.nodes.values().map(|node| node.label.as_str());
    let groups = graph.subgraphs.iter().map(|group| group.label.as_str());
    let edges = graph.edges.iter().flat_map(|edge| {
        edge.label
            .iter()
            .chain(&edge.start_label)
            .chain(&edge.end_label)
            .map(String::as_str)
    });
    nodes.chain(groups).chain(edges)
}

fn reads_as_markup(label: &str) -> bool {
    tag_openings(label).any(|opening| !opens_a_line_break(opening)) || carries_an_escape(label)
}

/// What comes after each `<` that opens something shaped like a tag. A `<` stuck
/// to another `<` opens nothing: it is a `<<interface>>` annotation, which is
/// Mermaid's own, which mmdr draws properly and which is the most idiomatic
/// thing a class diagram has.
fn tag_openings(label: &str) -> impl Iterator<Item = &str> {
    label.match_indices('<').filter_map(|(at, angle)| {
        let after = &label[at + angle.len()..];
        let annotation = label[..at].ends_with('<') || after.starts_with('<');
        (!annotation && looks_like_a_tag(after)).then_some(after)
    })
}

/// `<`, an optional slash, a name, and after it the close, the slash or an
/// attribute. `Map<String,Int>` has none —a comma is not a name—, and it is
/// drawn as written, which is the right thing.
fn looks_like_a_tag(after_the_angle: &str) -> bool {
    let name = after_the_angle.strip_prefix('/').unwrap_or(after_the_angle);
    let Some(at) = name.find(|character: char| !character.is_ascii_alphanumeric()) else {
        return false;
    };
    let rest = &name[at..];
    at > 0 && (rest.starts_with(['>', '/']) || rest.starts_with(char::is_whitespace))
}

/// The two mmdr interprets, and they are exactly two: measured, `<br />` with a
/// space, `<br  />` and `<BR/>` already come out literal.
fn opens_a_line_break(after_the_angle: &str) -> bool {
    after_the_angle.starts_with("br>") || after_the_angle.starts_with("br/>")
}

/// An HTML entity (`&amp;`, `&#35;`) or one of Mermaid's escapes (`#quot;`,
/// `#35;`). All three reach the drawing exactly as they were written.
fn carries_an_escape(label: &str) -> bool {
    label
        .char_indices()
        .filter(|(_, character)| matches!(character, '&' | '#'))
        .any(|(at, sign)| names_an_escape(&label[at + sign.len_utf8()..]))
}

fn names_an_escape(after_the_sign: &str) -> bool {
    let name = after_the_sign.strip_prefix('#').unwrap_or(after_the_sign);
    name.find(';').is_some_and(|at| {
        at > 0
            && name[..at]
                .chars()
                .all(|character| character.is_ascii_alphanumeric())
    })
}

/// mmdr **does not tell `flowchart TB` from `flowchart`** —the parser
/// initialises to `TopDown` and then overwrites—, so the question is put to the
/// source.
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
        let mut parsed = parse_mermaid(source).expect("the test source parses");
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
    fn the_classdef_is_emptied_out_of_the_ir() {
        let graph =
            graph("flowchart TD\n  classDef danger fill:#f00\n  A[One]:::danger --> B[Two]\n");

        assert!(graph.class_defs.is_empty());
        assert!(graph.node_classes.is_empty());
    }

    #[test]
    fn a_nodes_style_is_emptied_out_of_the_ir() {
        let graph = graph("flowchart TD\n  A[One] --> B[Two]\n  style A fill:#f00\n");

        assert!(graph.node_styles.is_empty());
    }

    #[test]
    fn the_linkstyle_is_emptied_out_of_the_ir() {
        let graph = graph("flowchart TD\n  A[One] --> B[Two]\n  linkStyle 0 stroke:#f00\n");

        assert!(graph.edge_styles.is_empty() && graph.edge_style_default.is_none());
    }

    #[test]
    fn the_click_is_emptied_out_of_the_ir() {
        let graph =
            graph("flowchart TD\n  A[One] --> B[Two]\n  click A \"https://example\" \"Go\"\n");

        assert!(graph.node_links.is_empty());
    }

    #[test]
    fn the_init_is_emptied_out_of_the_parse_output() {
        let (parsed, _) =
            imposed("%%{init: {'theme':'dark'}}%%\nflowchart TD\n  A[One] --> B[Two]\n");

        assert!(parsed.init_config.is_none());
    }

    #[test]
    fn the_dropped_style_carries_a_single_note_for_everything() {
        let notes = notes(
            "%%{init: {'theme':'dark'}}%%\nflowchart LR\n  classDef danger fill:#f00\n  \
             A[One]:::danger --> B[Two]\n  style B fill:#0f0\n  \
             click A \"https://example\" \"Go\"\n",
        );

        assert_eq!(notes, vec![STYLE_DROPPED]);
    }

    #[test]
    fn a_classdef_nobody_uses_warns_all_the_same_because_the_agent_believed_it_was_painting() {
        let notes = notes("flowchart LR\n  classDef danger fill:#f00\n  A[One] --> B[Two]\n");

        assert_eq!(notes, vec![STYLE_DROPPED]);
    }

    #[test]
    fn a_diagram_with_no_style_does_not_pay_the_note() {
        assert!(notes("flowchart LR\n  A[One] --> B[Two]\n").is_empty());
    }

    #[test]
    fn the_direction_is_imposed_on_the_flowchart() {
        assert_eq!(
            graph("flowchart TB\n  A[One] --> B[Two]\n").direction,
            Direction::LeftRight
        );
    }

    #[test]
    fn the_direction_is_imposed_on_the_class_diagram() {
        assert_eq!(
            graph("classDiagram\n  direction TB\n  Order <|-- Sale\n").direction,
            Direction::LeftRight
        );
    }

    #[test]
    fn the_imposition_reaches_down_to_each_groups_direction() {
        let graph =
            graph("flowchart TD\n  subgraph API\n    direction RL\n    A[One] --> B[Two]\n  end\n");

        assert!(
            graph
                .subgraphs
                .iter()
                .all(|group| group.direction.is_none())
        );
    }

    #[test]
    fn outside_the_two_families_the_direction_is_not_touched() {
        let graph = graph("stateDiagram-v2\n  direction TB\n  [*] --> Active\n");

        assert_eq!(graph.direction, Direction::TopDown);
    }

    #[test]
    fn a_declared_direction_that_differs_is_warned_about() {
        assert_eq!(
            notes("flowchart TB\n  A[One] --> B[Two]\n"),
            vec![DIRECTION_IMPOSED]
        );
    }

    #[test]
    fn a_bare_header_declared_no_direction_and_is_not_paid_for() {
        assert!(notes("flowchart\n  A[One] --> B[Two]\n").is_empty());
    }

    #[test]
    fn a_header_that_already_asked_for_lr_is_not_warned_about() {
        assert!(notes("flowchart LR\n  A[One] --> B[Two]\n").is_empty());
    }

    #[test]
    fn a_groups_direction_is_also_a_declared_direction() {
        let notes =
            notes("flowchart LR\n  subgraph API\n    direction TB\n    A[One] --> B[Two]\n  end\n");

        assert_eq!(notes, vec![DIRECTION_IMPOSED]);
    }

    #[test]
    fn outside_the_two_families_a_declared_direction_does_not_warn() {
        assert!(notes("stateDiagram-v2\n  direction TB\n  [*] --> Active\n").is_empty());
    }

    #[test]
    fn the_namespace_mmdr_cannot_draw_is_warned_about() {
        let notes = notes("classDiagram\n  namespace Domain {\n    class Order\n  }\n");

        assert_eq!(notes, vec![NAMESPACE_NOT_DRAWN]);
    }

    #[test]
    fn the_note_mmdr_cannot_draw_is_warned_about() {
        let notes = notes("classDiagram\n  note for Order \"the note\"\n  class Order\n");

        assert_eq!(notes, vec![NOTES_NOT_DRAWN]);
    }

    #[test]
    fn a_namespace_outside_the_class_diagram_is_not_warned_about() {
        assert!(notes("flowchart LR\n  namespace --> B[Two]\n").is_empty());
    }

    #[test]
    fn the_line_break_mmdr_interprets_warns_about_nothing() {
        assert!(
            notes("flowchart LR\n  store[\"storelua<br/>persistence\"] --> marks[\"markslua<br>repositioning\"]\n")
                .is_empty()
        );
    }

    #[test]
    fn the_bold_that_ends_up_drawn_as_text_is_warned_about() {
        let notes =
            notes("flowchart LR\n  store[\"storelua\"] --> marks[\"<b>repositioning</b>\"]\n");

        assert_eq!(notes, vec![MARKUP_DRAWN_AS_TEXT]);
    }

    #[test]
    fn the_line_break_with_a_space_inside_already_comes_out_literal_and_is_warned_about() {
        let notes =
            notes("flowchart LR\n  store[\"storelua<br />persistence\"] --> marks[\"markslua\"]\n");

        assert_eq!(notes, vec![MARKUP_DRAWN_AS_TEXT]);
    }

    #[test]
    fn mmdr_does_not_interpret_the_line_break_in_capitals_either() {
        let notes =
            notes("flowchart LR\n  store[\"storelua<BR/>persistence\"] --> marks[\"markslua\"]\n");

        assert_eq!(notes, vec![MARKUP_DRAWN_AS_TEXT]);
    }

    #[test]
    fn the_whole_family_of_escaped_labels_warns_all_the_same() {
        for label in [
            "<i>one</i>",
            "<em>one</em>",
            "<strong>one</strong>",
            "<u>one</u>",
            "<code>one</code>",
            "<span style='color:red'>one</span>",
            "<a href='https://example'>one</a>",
            "<img src='x.png'/>one",
        ] {
            let notes = notes(&format!(
                "flowchart LR\n  store[\"{label}\"] --> marks[\"markslua\"]\n"
            ));

            assert_eq!(notes, vec![MARKUP_DRAWN_AS_TEXT], "fails with {label}");
        }
    }

    #[test]
    fn entities_and_mermaids_escapes_arrive_as_text_and_are_warned_about() {
        for label in [
            "storelua &amp; marks",
            "storelua&nbsp;persistence",
            "&lt;storelua&gt;",
            "storelua &#35; persistence",
            "storelua #quot;persistence#quot;",
            "storelua #35; persistence",
        ] {
            let notes = notes(&format!(
                "flowchart LR\n  store[\"{label}\"] --> marks[\"markslua\"]\n"
            ));

            assert_eq!(notes, vec![MARKUP_DRAWN_AS_TEXT], "fails with {label}");
        }
    }

    #[test]
    fn the_raw_characters_mmdr_escapes_properly_do_not_warn() {
        for label in [
            "storelua & persistence",
            "storelua < persistence",
            "the usual 'storelua'",
            "Map<String,Int>",
        ] {
            assert!(
                notes(&format!(
                    "flowchart LR\n  store[\"{label}\"] --> marks[\"markslua\"]\n"
                ))
                .is_empty(),
                "over-warns on {label}"
            );
        }
    }

    #[test]
    fn the_class_diagrams_annotation_is_not_markup() {
        assert!(
            notes("classDiagram\n  class Order {\n    <<interface>>\n    +confirm() void\n  }\n")
                .is_empty()
        );
    }

    #[test]
    fn markup_in_an_edges_label_is_warned_about_too() {
        let notes = notes(
            "flowchart LR\n  store[\"storelua\"] -->|\"<b>reads</b>\"| marks[\"markslua\"]\n",
        );

        assert_eq!(notes, vec![MARKUP_DRAWN_AS_TEXT]);
    }

    #[test]
    fn markup_in_a_groups_label_is_warned_about_too() {
        let notes = notes(
            "flowchart LR\n  subgraph layer[\"<b>Data</b> layer\"]\n    store[\"storelua\"]\n  \
             end\n  store --> marks[\"markslua\"]\n",
        );

        assert_eq!(notes, vec![MARKUP_DRAWN_AS_TEXT]);
    }

    #[test]
    fn markup_in_a_class_member_is_warned_about_too() {
        let notes = notes("classDiagram\n  class Order {\n    +<b>confirm</b>() void\n  }\n");

        assert_eq!(notes, vec![MARKUP_DRAWN_AS_TEXT]);
    }

    #[test]
    fn the_line_break_in_a_class_member_comes_out_right_and_does_not_warn() {
        assert!(notes("classDiagram\n  class Order {\n    +confirm()<br/>void\n  }\n").is_empty());
    }

    #[test]
    fn the_notes_accumulate() {
        let notes = notes(
            "%%{init: {'theme':'dark'}}%%\nclassDiagram\n  direction TB\n  \
             namespace Domain {\n    class Order\n  }\n  note \"the note\"\n",
        );

        assert_eq!(
            notes,
            vec![
                STYLE_DROPPED,
                NAMESPACE_NOT_DRAWN,
                NOTES_NOT_DRAWN,
                DIRECTION_IMPOSED
            ]
        );
    }

    #[test]
    fn a_comment_declares_neither_direction_nor_structure() {
        assert!(
            notes("classDiagram\n  %% direction TB\n  %% namespace Domain\n  class Order\n")
                .is_empty()
        );
    }
}
