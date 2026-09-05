#![allow(
    clippy::print_stdout,
    reason = "the diagnostic subcommand owns stdout: there is no MCP session to corrupt"
)]

use std::path::Path;

use crate::diagram;

/// Runs the pipeline over `.mmd` files and prints the outcome and its text,
/// without opening a window. It is what makes the Honest limit's rules
/// measurable.
pub fn check(paths: &[String]) {
    for path in paths {
        println!("== {path}");
        match std::fs::read_to_string(path) {
            Err(error) => println!("unreadable\n{error}"),
            Ok(source) => println!("{}", outcome_of(&source, view_id(path))),
        }
    }
}

fn outcome_of(source: &str, view_id: &str) -> String {
    match diagram::draw(source) {
        Ok(drawing) => drawing.noted_after(format!("drawn\n{}", drawing.recount())),
        Err(rejection) => format!("{}\n{}", rejection.outcome(), rejection.told_about(view_id)),
    }
}

fn view_id(path: &str) -> &str {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_diagram_that_gets_drawn_says_the_outcome_and_the_recount() {
        let text = outcome_of("flowchart LR\n  A[One] --> B[Two]\n", "two-nodes");

        assert_eq!(text, "drawn\n2 nodes, 1 edge");
    }

    #[test]
    fn a_drawing_with_a_note_prints_it_after_the_recount() {
        let text = outcome_of("flowchart TB\n  A[One] --> B[Two]\n", "two-nodes");

        assert_eq!(
            text,
            "drawn\n2 nodes, 1 edge\n\
             Note: the flipchart lays diagrams out left to right; the direction in your \
             source was ignored. The view was drawn."
        );
    }

    #[test]
    fn a_rejection_says_the_outcome_and_the_text_the_agent_would_receive() {
        let text = outcome_of("flowchart TD\n  API[API Layer] --> Db\n", "fc-99");

        assert_eq!(
            text,
            "undeclared nodes\n\
             Rejected: nothing was drawn; view \"fc-99\" is unchanged.\n\
             1 node appears in the drawing that you did not declare.\n  \
             \"Db\"  line 2  — only used in a relation\n\
             Declare every node you name, and rewrite any line the renderer turned into one."
        );
    }

    #[test]
    fn the_diagnostic_view_id_is_the_file_name() {
        assert_eq!(view_id("cases/fc-11-bare-header.mmd"), "fc-11-bare-header");
    }
}
