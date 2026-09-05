use std::panic::{AssertUnwindSafe, catch_unwind};

use mermaid_rs_renderer::{
    LayoutConfig, ParseError, ParseOutput, Theme, compute_layout, parse_mermaid,
    parse_mermaid_strict, render_svg,
};

mod honest_limit;
mod house_style;

use self::honest_limit::undeclared_nodes;

#[derive(Debug)]
pub struct Drawing {
    pub svg: String,
    pub nodes: usize,
    pub edges: usize,
    pub notes: Vec<&'static str>,
}

impl Drawing {
    /// The only feedback the agent has about the drawing, because the image
    /// never comes back into the context.
    pub fn recount(&self) -> String {
        format!(
            "{}, {}",
            plural(self.nodes, "node"),
            plural(self.edges, "edge")
        )
    }

    /// The notes go after, one per line: the agent reads the outcome first and
    /// the price afterwards.
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

/// The four outcomes of `show` that do not draw. The panic one is the only one
/// that says the fault is ours, on purpose: if we ask the agent to fix its
/// diagram, it will try in a loop on something that has no fix.
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

/// Factory settings, untouched: no mmdr knob improves the drawing and several
/// do harm. Measured in ADR-0003.
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

/// Server and Viewer share a process, so an uncaught panic would take the whole
/// flipchart down with it, silently.
fn guarded<T>(step: impl FnOnce() -> T) -> Result<T, Rejection> {
    catch_unwind(AssertUnwindSafe(step)).map_err(|_| Rejection::RendererPanicked)
}

/// We come in through the permissive path; strict is only paid for when nothing
/// is going to be drawn any more, and only to get the typed `ParseError` for the
/// message.
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

    const TWO_NODES: &str = "flowchart TD\n  A[One] --> B[Two]\n";

    /// The **leading case**, which is understanding a refactor before doing it:
    /// four groups and seven edges.
    fn arch() -> String {
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cases/arch.mmd"))
            .expect("the leading case is in the repo")
    }

    #[test]
    fn the_pipeline_returns_an_svg_with_the_labels_inside() {
        let drawing = draw(TWO_NODES).unwrap();

        assert!(drawing.svg.starts_with("<svg"));
        assert!(drawing.svg.contains("One"));
        assert!(drawing.svg.contains("Two"));
    }

    #[test]
    fn the_pipeline_counts_what_it_has_drawn() {
        let drawing = draw(TWO_NODES).unwrap();

        assert_eq!((drawing.nodes, drawing.edges), (2, 1));
    }

    #[test]
    fn the_theme_is_mermaids_and_not_the_modern_14_px_one() {
        assert_eq!(theme().font_size, Theme::mermaid_default().font_size);
        assert_ne!(theme().font_size, Theme::modern().font_size);
    }

    #[test]
    fn what_is_not_mermaid_never_gets_drawn() {
        assert!(draw("this is not Mermaid").is_err());
    }

    #[test]
    fn the_leading_case_comes_out_whole_through_the_pipeline() {
        let drawing = draw(&arch()).unwrap();

        assert_eq!((drawing.nodes, drawing.edges), (8, 7));
        for group in ["API", "Application", "Domain", "Infrastructure"] {
            assert!(drawing.svg.contains(group), "the {group} group is missing");
        }
    }

    #[test]
    fn what_does_not_parse_is_the_parse_error_outcome() {
        let rejection = draw("this is not Mermaid").unwrap_err();

        assert_eq!(rejection.outcome(), "parse error");
    }

    #[test]
    fn a_node_the_agent_did_not_declare_is_the_rules_outcome() {
        let rejection = draw("flowchart TD\n  API[API Layer] --> Db\n").unwrap_err();

        assert_eq!(rejection.outcome(), "undeclared nodes");
    }

    #[test]
    fn the_parse_error_comes_before_the_rules_because_without_a_graph_there_are_none() {
        let rejection = draw("not Mermaid\n  API[API Layer] --> Db\n").unwrap_err();

        assert_eq!(rejection.outcome(), "parse error");
    }

    #[test]
    fn every_rejection_that_is_not_the_panic_opens_with_the_fixed_line() {
        let rejection = draw("this is not Mermaid").unwrap_err();

        assert!(
            rejection
                .told_about("proposed")
                .starts_with("Rejected: nothing was drawn; view \"proposed\" is unchanged.\n")
        );
    }

    #[test]
    fn the_panic_is_the_only_rejection_that_says_the_fault_is_ours() {
        let text = Rejection::RendererPanicked.told_about("proposed");

        assert_eq!(
            text,
            "Rejected: the renderer failed on this diagram; nothing was drawn.\n\
             View \"proposed\" is unchanged. This is a bug in the flipchart, not in your \
             diagram — try a simpler diagram, or the same one with fewer nodes."
        );
    }

    #[test]
    fn the_unexpected_token_carries_line_column_and_what_was_found() {
        let text = diagnosed(&ParseError::UnexpectedToken {
            line: 4,
            col: 3,
            found: "-->".to_string(),
            expected: "class".to_string(),
        });

        assert_eq!(
            text,
            "Unexpected token at line 4, column 3 — found \"-->\", expected class."
        );
    }

    #[test]
    fn a_long_expected_stops_at_the_first_three() {
        let text = diagnosed(&ParseError::UnexpectedToken {
            line: 4,
            col: 3,
            found: "-->".to_string(),
            expected: "class, }, an identifier, a comment".to_string(),
        });

        assert!(text.ends_with("expected class, } or an identifier."));
    }

    #[test]
    fn the_unknown_participant_offers_the_candidates() {
        let text = diagnosed(&ParseError::UnknownParticipant {
            name: "Ordr".to_string(),
            line: 6,
            candidates: vec!["Order".to_string()],
        });

        assert_eq!(
            text,
            "Unknown node \"Ordr\" at line 6 — did you mean \"Order\"?"
        );
    }

    #[test]
    fn the_unknown_participant_with_no_candidates_invents_none() {
        let text = diagnosed(&ParseError::UnknownParticipant {
            name: "Ordr".to_string(),
            line: 6,
            candidates: Vec::new(),
        });

        assert_eq!(
            text,
            "Unknown node \"Ordr\" at line 6 — it was never declared."
        );
    }

    #[test]
    fn the_unclosed_subgraph_says_where_it_was_opened() {
        let text = diagnosed(&ParseError::UnclosedSubgraph { opened_at: 3 });

        assert_eq!(text, "A subgraph opened at line 3 was never closed.");
    }

    #[test]
    fn the_unclassified_variant_admits_we_have_not_classified_it() {
        let text = diagnosed(&ParseError::InvalidDirective {
            line: 2,
            col: 1,
            directive: "init".to_string(),
            reason: "empty body".to_string(),
        });

        assert!(text.starts_with("The flipchart did not classify this failure: "));
    }
}
