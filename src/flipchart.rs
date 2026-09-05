use crate::diagram::{self, Rejection};
use crate::viewer::{DeckSnapshot, Drawn, Wire};

const MAX_VIEW_ID_CHARS: usize = 64;

/// A live View. `drawn` numbers it at the moment it was drawn, so **the highest
/// is the one from the most recent live `show`**: the one at the front.
#[derive(Debug)]
struct View {
    id: String,
    diagram: String,
    svg: String,
    drawn: u64,
}

#[derive(Debug)]
pub struct Flipchart {
    views: Vec<View>,
    drawn: u64,
    viewer: Wire,
}

impl Flipchart {
    pub fn new(viewer: Wire) -> Self {
        Self {
            views: Vec::new(),
            drawn: 0,
            viewer,
        }
    }

    pub fn show(&mut self, view_id: &str, diagram: &str) -> Result<String, String> {
        let id = view_id.trim();
        self.draw(id, diagram)
            .map_err(|rejection| rejection.told_about(id))
    }

    fn draw(&mut self, id: &str, source: &str) -> Result<String, Rejection> {
        if id.is_empty() {
            return Err(Rejection::InvalidInput(
                "view_id must not be empty.".to_string(),
            ));
        }
        if id.chars().count() > MAX_VIEW_ID_CHARS {
            return Err(Rejection::InvalidInput(format!(
                "view_id must be at most {MAX_VIEW_ID_CHARS} characters; got {}.",
                id.chars().count()
            )));
        }
        if source.trim().is_empty() {
            return Err(Rejection::InvalidInput(
                "diagram must not be empty.".to_string(),
            ));
        }

        let drawing = diagram::draw(source)?;
        let recount = drawing.recount();

        self.drawn += 1;
        let drawn = self.drawn;
        match self.views.iter_mut().find(|view| view.id == id) {
            Some(view) => {
                view.diagram = source.to_string();
                view.svg = drawing.svg.clone();
                view.drawn = drawn;
            }
            None => self.views.push(View {
                id: id.to_string(),
                diagram: source.to_string(),
                svg: drawing.svg.clone(),
                drawn,
            }),
        }
        let acknowledgement = drawing.noted_after(format!(
            "Shown as view \"{id}\" ({}). {}",
            recount,
            self.views_on_the_flipchart()
        ));
        self.hand_the_deck_over();

        Ok(acknowledgement)
    }

    pub fn clear(&mut self, view_id: Option<&str>) -> String {
        let Some(id) = view_id else {
            if self.views.is_empty() {
                return "The flipchart was already empty.".to_string();
            }
            self.views.clear();
            self.hand_the_deck_over();
            return "Cleared the flipchart. No views.".to_string();
        };

        let Some(position) = self.views.iter().position(|view| view.id == id) else {
            return format!("No view \"{id}\" on the flipchart. {}", self.views());
        };
        self.views.remove(position);
        self.hand_the_deck_over();
        format!("Cleared view \"{id}\". {}", self.views_on_the_flipchart())
    }

    /// The order is the order of creation and the front one is the one from the
    /// most recent live `show`. Both are decided here by the MCP server, not by
    /// the Viewer.
    fn hand_the_deck_over(&mut self) {
        self.viewer.send(DeckSnapshot {
            sheets: self
                .views
                .iter()
                .map(|view| Drawn {
                    number: view.drawn,
                    id: view.id.clone(),
                    svg: view.svg.clone(),
                })
                .collect(),
            front: self.front(),
        });
    }

    fn front(&self) -> Option<usize> {
        self.views
            .iter()
            .enumerate()
            .max_by_key(|(_, view)| view.drawn)
            .map(|(position, _)| position)
    }

    pub fn view(&self, view_id: &str) -> Option<&str> {
        self.views
            .iter()
            .find(|view| view.id == view_id)
            .map(|view| view.diagram.as_str())
    }

    fn views_on_the_flipchart(&self) -> String {
        match self.view_ids() {
            Some(ids) => format!("Views on the flipchart: {ids}."),
            None => "No views.".to_string(),
        }
    }

    fn views(&self) -> String {
        match self.view_ids() {
            Some(ids) => format!("Views: {ids}."),
            None => "No views.".to_string(),
        }
    }

    fn view_ids(&self) -> Option<String> {
        if self.views.is_empty() {
            return None;
        }
        Some(
            self.views
                .iter()
                .map(|view| view.id.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewer::{Command, Commands, wire};

    const TWO_NODES: &str = "flowchart LR\n  A[One] --> B[Two]\n";
    const THREE_NODES: &str = "flowchart LR\n  A[One] --> B[Two]\n  B --> C[Three]\n";
    const THREE_NODES_DOWNWARDS: &str = "flowchart TB\n  A[One] --> B[Two]\n  B --> C[Three]\n";
    const ONE_NODE: &str = "flowchart LR\n  A[Alone]\n";

    fn flipchart() -> (Flipchart, Commands) {
        let (viewer, commands) = wire();
        (Flipchart::new(viewer), commands)
    }

    fn handed_over(commands: &Commands) -> DeckSnapshot {
        let mut last = None;
        while let Some(Command::Show(snapshot)) = commands.try_recv() {
            last = Some(snapshot);
        }
        last.expect("a flipchart reached the Viewer")
    }

    fn names(snapshot: &DeckSnapshot) -> Vec<&str> {
        snapshot
            .sheets
            .iter()
            .map(|sheet| sheet.id.as_str())
            .collect()
    }

    fn front(snapshot: &DeckSnapshot) -> &str {
        let front = snapshot.front.expect("there is a sheet at the front");
        &snapshot.sheets[front].id
    }

    #[test]
    fn the_acknowledgement_carries_the_id_the_recount_and_the_live_views() {
        let (mut flipchart, _commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();

        let acknowledgement = flipchart.show("proposed", THREE_NODES).unwrap();

        assert_eq!(
            acknowledgement,
            "Shown as view \"proposed\" (3 nodes, 2 edges). \
             Views on the flipchart: current, proposed."
        );
    }

    #[test]
    fn the_recount_goes_singular_when_there_is_one_of_each() {
        let (mut flipchart, _commands) = flipchart();

        let acknowledgement = flipchart.show("alone", TWO_NODES);

        assert!(acknowledgement.unwrap().contains("(2 nodes, 1 edge)"));
    }

    #[test]
    fn a_view_with_no_edges_counts_zero() {
        let (mut flipchart, _commands) = flipchart();

        let acknowledgement = flipchart.show("alone", ONE_NODE);

        assert!(acknowledgement.unwrap().contains("(1 node, 0 edges)"));
    }

    #[test]
    fn reusing_an_id_replaces_the_view_without_moving_it() {
        let (mut flipchart, _commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();
        flipchart.show("proposed", TWO_NODES).unwrap();

        let acknowledgement = flipchart.show("current", THREE_NODES).unwrap();

        assert!(acknowledgement.ends_with("Views on the flipchart: current, proposed."));
    }

    #[test]
    fn reusing_an_id_keeps_the_new_diagram() {
        let (mut flipchart, _commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();

        flipchart.show("current", THREE_NODES).unwrap();

        assert_eq!(flipchart.view("current"), Some(THREE_NODES));
    }

    #[test]
    fn the_viewer_learns_about_the_shown_view() {
        let (mut flipchart, commands) = flipchart();

        flipchart.show("current", TWO_NODES).unwrap();

        assert_eq!(names(&handed_over(&commands)), ["current"]);
    }

    #[test]
    fn what_reaches_the_viewer_is_the_already_drawn_svg() {
        let (mut flipchart, commands) = flipchart();

        flipchart.show("current", TWO_NODES).unwrap();

        let svg = &handed_over(&commands).sheets[0].svg;
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("One"));
    }

    #[test]
    fn the_sheets_arrive_in_creation_order_and_replacing_does_not_reorder() {
        let (mut flipchart, commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();
        flipchart.show("proposed", TWO_NODES).unwrap();

        flipchart.show("current", THREE_NODES).unwrap();

        assert_eq!(names(&handed_over(&commands)), ["current", "proposed"]);
    }

    #[test]
    fn a_show_leaves_its_view_at_the_front() {
        let (mut flipchart, commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();
        flipchart.show("proposed", TWO_NODES).unwrap();

        flipchart.show("current", THREE_NODES).unwrap();

        assert_eq!(front(&handed_over(&commands)), "current");
    }

    #[test]
    fn replacing_a_view_sends_a_new_sheet() {
        let (mut flipchart, commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();
        let first = handed_over(&commands).sheets[0].number;

        flipchart.show("current", THREE_NODES).unwrap();

        assert_ne!(handed_over(&commands).sheets[0].number, first);
    }

    #[test]
    fn removing_the_front_view_falls_back_to_the_most_recent_live_show() {
        let (mut flipchart, commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();
        flipchart.show("proposed", TWO_NODES).unwrap();
        flipchart.show("flow", TWO_NODES).unwrap();

        flipchart.clear(Some("flow"));

        assert_eq!(front(&handed_over(&commands)), "proposed");
    }

    #[test]
    fn removing_another_view_leaves_at_the_front_the_one_already_there() {
        let (mut flipchart, commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();
        flipchart.show("proposed", TWO_NODES).unwrap();

        flipchart.clear(Some("current"));

        assert_eq!(front(&handed_over(&commands)), "proposed");
    }

    #[test]
    fn emptying_the_flipchart_hands_over_a_flipchart_with_no_sheets() {
        let (mut flipchart, commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();

        flipchart.clear(None);

        assert!(handed_over(&commands).sheets.is_empty());
    }

    #[test]
    fn a_flipchart_with_no_sheets_has_none_at_the_front() {
        let (mut flipchart, commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();

        flipchart.clear(None);

        assert_eq!(handed_over(&commands).front, None);
    }

    #[test]
    fn clearing_an_id_that_does_not_exist_tells_the_viewer_nothing() {
        let (mut flipchart, commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();
        handed_over(&commands);

        flipchart.clear(Some("propsoed"));

        assert!(commands.try_recv().is_none());
    }

    #[test]
    fn a_blank_view_id_is_rejected() {
        let (mut flipchart, _commands) = flipchart();

        let rejection = flipchart.show("   ", TWO_NODES).unwrap_err();

        assert!(rejection.contains("view_id must not be empty."));
    }

    #[test]
    fn a_view_id_longer_than_64_characters_is_rejected() {
        let (mut flipchart, _commands) = flipchart();

        let rejection = flipchart.show(&"a".repeat(65), TWO_NODES).unwrap_err();

        assert!(rejection.contains("view_id must be at most 64 characters; got 65."));
    }

    #[test]
    fn a_view_id_of_64_characters_goes_in() {
        let (mut flipchart, _commands) = flipchart();

        let acknowledgement = flipchart.show(&"a".repeat(64), TWO_NODES);

        assert!(acknowledgement.is_ok());
    }

    #[test]
    fn the_view_id_is_prose_and_not_a_slug() {
        let (mut flipchart, _commands) = flipchart();

        let acknowledgement = flipchart
            .show("Current structure (v2) — really?", TWO_NODES)
            .unwrap();

        assert!(acknowledgement.starts_with("Shown as view \"Current structure (v2) — really?\""));
    }

    #[test]
    fn the_view_id_is_stored_trimmed() {
        let (mut flipchart, _commands) = flipchart();

        let acknowledgement = flipchart.show("  current  ", TWO_NODES).unwrap();

        assert!(acknowledgement.starts_with("Shown as view \"current\""));
    }

    #[test]
    fn an_empty_diagram_is_rejected() {
        let (mut flipchart, _commands) = flipchart();

        let rejection = flipchart.show("current", "  \n ").unwrap_err();

        assert!(rejection.contains("diagram must not be empty."));
    }

    #[test]
    fn the_input_is_validated_before_parsing() {
        let (mut flipchart, _commands) = flipchart();

        let rejection = flipchart.show("", "this is not Mermaid").unwrap_err();

        assert!(rejection.contains("view_id must not be empty."));
    }

    #[test]
    fn a_rejection_leaves_the_view_that_was_already_there_intact() {
        let (mut flipchart, _commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();

        flipchart.show("current", "").unwrap_err();

        assert_eq!(flipchart.view("current"), Some(TWO_NODES));
    }

    #[test]
    fn the_rejection_opens_with_the_fixed_line() {
        let (mut flipchart, _commands) = flipchart();

        let rejection = flipchart.show("current", "").unwrap_err();

        assert!(
            rejection.starts_with("Rejected: nothing was drawn; view \"current\" is unchanged.\n")
        );
    }

    #[test]
    fn clearing_a_view_says_so_and_lists_what_is_left() {
        let (mut flipchart, _commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();
        flipchart.show("proposed", TWO_NODES).unwrap();

        let text = flipchart.clear(Some("proposed"));

        assert_eq!(
            text,
            "Cleared view \"proposed\". Views on the flipchart: current."
        );
    }

    #[test]
    fn clearing_the_last_view_leaves_the_flipchart_with_no_views() {
        let (mut flipchart, _commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();

        let text = flipchart.clear(Some("current"));

        assert_eq!(text, "Cleared view \"current\". No views.");
    }

    #[test]
    fn clearing_the_whole_flipchart_says_so() {
        let (mut flipchart, _commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();

        let text = flipchart.clear(None);

        assert_eq!(text, "Cleared the flipchart. No views.");
    }

    #[test]
    fn clearing_an_id_that_does_not_exist_is_not_an_error_and_carries_the_list_alongside() {
        let (mut flipchart, _commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();
        flipchart.show("proposed", TWO_NODES).unwrap();

        let text = flipchart.clear(Some("propsoed"));

        assert_eq!(
            text,
            "No view \"propsoed\" on the flipchart. Views: current, proposed."
        );
    }

    #[test]
    fn clearing_an_id_that_does_not_exist_does_not_touch_the_views() {
        let (mut flipchart, _commands) = flipchart();
        flipchart.show("current", TWO_NODES).unwrap();

        flipchart.clear(Some("propsoed"));

        assert_eq!(flipchart.view("current"), Some(TWO_NODES));
    }

    #[test]
    fn clearing_an_already_empty_flipchart_says_so() {
        let (mut flipchart, _commands) = flipchart();

        let text = flipchart.clear(None);

        assert_eq!(text, "The flipchart was already empty.");
    }

    #[test]
    fn the_acknowledgement_drags_the_notes_behind_it() {
        let (mut flipchart, _commands) = flipchart();

        let acknowledgement = flipchart.show("current", THREE_NODES_DOWNWARDS).unwrap();

        assert_eq!(
            acknowledgement,
            "Shown as view \"current\" (3 nodes, 2 edges). Views on the flipchart: current.\n\
             Note: the flipchart lays diagrams out left to right; the direction in your \
             source was ignored. The view was drawn."
        );
    }

    #[test]
    fn a_rejection_carries_no_notes() {
        let (mut flipchart, _commands) = flipchart();

        let rejection = flipchart
            .show(
                "current",
                "flowchart TB\n  classDef danger fill:#f00\n  A[One] --> Db\n",
            )
            .unwrap_err();

        assert!(!rejection.contains("Note:"));
    }

    #[test]
    fn a_diagram_that_does_not_parse_is_rejected_without_touching_the_flipchart() {
        let (mut flipchart, _commands) = flipchart();

        let rejection = flipchart
            .show("current", "this is not Mermaid")
            .unwrap_err();

        assert!(rejection.starts_with("Rejected: nothing was drawn;"));
        assert_eq!(flipchart.view("current"), None);
    }
}
