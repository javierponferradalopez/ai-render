use crate::diagram::{self, Rejection};
use crate::viewer::{ViewerCommand, Wire};

const MAX_VIEW_ID_CHARS: usize = 64;

#[derive(Debug)]
struct View {
    id: String,
    diagram: String,
}

#[derive(Debug)]
pub struct Flipchart {
    views: Vec<View>,
    viewer: Wire,
}

impl Flipchart {
    pub fn new(viewer: Wire) -> Self {
        Self {
            views: Vec::new(),
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

        match self.views.iter_mut().find(|view| view.id == id) {
            Some(view) => view.diagram = source.to_string(),
            None => self.views.push(View {
                id: id.to_string(),
                diagram: source.to_string(),
            }),
        }
        let acknowledgement = drawing.noted_after(format!(
            "Shown as view \"{id}\" ({}). {}",
            recount,
            self.views_on_the_flipchart()
        ));
        self.viewer.send(ViewerCommand::Show {
            view_id: id.to_string(),
            svg: drawing.svg,
        });

        Ok(acknowledgement)
    }

    pub fn clear(&mut self, view_id: Option<&str>) -> String {
        let Some(id) = view_id else {
            if self.views.is_empty() {
                return "The flipchart was already empty.".to_string();
            }
            self.views.clear();
            self.viewer.send(ViewerCommand::Clear);
            return "Cleared the flipchart. No views.".to_string();
        };

        let Some(position) = self.views.iter().position(|view| view.id == id) else {
            return format!("No view \"{id}\" on the flipchart. {}", self.views());
        };
        self.views.remove(position);
        self.viewer.send(ViewerCommand::Clear);
        format!("Cleared view \"{id}\". {}", self.views_on_the_flipchart())
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
    use crate::viewer::{Commands, wire};

    const DOS_NODOS: &str = "flowchart LR\n  A[Uno] --> B[Dos]\n";
    const TRES_NODOS: &str = "flowchart LR\n  A[Uno] --> B[Dos]\n  B --> C[Tres]\n";
    const TRES_NODOS_HACIA_ABAJO: &str = "flowchart TB\n  A[Uno] --> B[Dos]\n  B --> C[Tres]\n";
    const UN_NODO: &str = "flowchart LR\n  A[Solo]\n";

    fn pizarra() -> (Flipchart, Commands) {
        let (viewer, commands) = wire();
        (Flipchart::new(viewer), commands)
    }

    #[test]
    fn el_acuse_lleva_el_id_el_recuento_y_las_vistas_vivas() {
        let (mut flipchart, _commands) = pizarra();
        flipchart.show("actual", DOS_NODOS).unwrap();

        let acuse = flipchart.show("propuesto", TRES_NODOS).unwrap();

        assert_eq!(
            acuse,
            "Shown as view \"propuesto\" (3 nodes, 2 edges). \
             Views on the flipchart: actual, propuesto."
        );
    }

    #[test]
    fn el_recuento_va_en_singular_cuando_hay_uno_de_cada() {
        let (mut flipchart, _commands) = pizarra();

        let acuse = flipchart.show("solo", DOS_NODOS);

        assert!(acuse.unwrap().contains("(2 nodes, 1 edge)"));
    }

    #[test]
    fn una_vista_sin_aristas_cuenta_cero() {
        let (mut flipchart, _commands) = pizarra();

        let acuse = flipchart.show("solo", UN_NODO);

        assert!(acuse.unwrap().contains("(1 node, 0 edges)"));
    }

    #[test]
    fn reusar_un_id_reemplaza_la_vista_sin_moverla_de_sitio() {
        let (mut flipchart, _commands) = pizarra();
        flipchart.show("actual", DOS_NODOS).unwrap();
        flipchart.show("propuesto", DOS_NODOS).unwrap();

        let acuse = flipchart.show("actual", TRES_NODOS).unwrap();

        assert!(acuse.ends_with("Views on the flipchart: actual, propuesto."));
    }

    #[test]
    fn reusar_un_id_se_queda_con_el_diagrama_nuevo() {
        let (mut flipchart, _commands) = pizarra();
        flipchart.show("actual", DOS_NODOS).unwrap();

        flipchart.show("actual", TRES_NODOS).unwrap();

        assert_eq!(flipchart.view("actual"), Some(TRES_NODOS));
    }

    #[test]
    fn el_visor_se_entera_de_la_vista_mostrada() {
        let (mut flipchart, commands) = pizarra();

        flipchart.show("actual", DOS_NODOS).unwrap();

        let ViewerCommand::Show { view_id, .. } = commands.recv().unwrap() else {
            panic!("el Visor esperaba un Show");
        };
        assert_eq!(view_id, "actual");
    }

    #[test]
    fn al_visor_le_cruza_el_svg_ya_dibujado() {
        let (mut flipchart, commands) = pizarra();

        flipchart.show("actual", DOS_NODOS).unwrap();

        let ViewerCommand::Show { svg, .. } = commands.recv().unwrap() else {
            panic!("el Visor esperaba un Show");
        };
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("Uno"));
    }

    #[test]
    fn un_view_id_en_blanco_se_rechaza() {
        let (mut flipchart, _commands) = pizarra();

        let rechazo = flipchart.show("   ", DOS_NODOS).unwrap_err();

        assert!(rechazo.contains("view_id must not be empty."));
    }

    #[test]
    fn un_view_id_de_mas_de_64_caracteres_se_rechaza() {
        let (mut flipchart, _commands) = pizarra();

        let rechazo = flipchart.show(&"a".repeat(65), DOS_NODOS).unwrap_err();

        assert!(rechazo.contains("view_id must be at most 64 characters; got 65."));
    }

    #[test]
    fn un_view_id_de_64_caracteres_entra() {
        let (mut flipchart, _commands) = pizarra();

        let acuse = flipchart.show(&"a".repeat(64), DOS_NODOS);

        assert!(acuse.is_ok());
    }

    #[test]
    fn el_view_id_es_prosa_y_no_un_slug() {
        let (mut flipchart, _commands) = pizarra();

        let acuse = flipchart
            .show("Estructura actual (v2), ¿sí?", DOS_NODOS)
            .unwrap();

        assert!(acuse.starts_with("Shown as view \"Estructura actual (v2), ¿sí?\""));
    }

    #[test]
    fn el_view_id_se_guarda_recortado() {
        let (mut flipchart, _commands) = pizarra();

        let acuse = flipchart.show("  actual  ", DOS_NODOS).unwrap();

        assert!(acuse.starts_with("Shown as view \"actual\""));
    }

    #[test]
    fn un_diagrama_vacio_se_rechaza() {
        let (mut flipchart, _commands) = pizarra();

        let rechazo = flipchart.show("actual", "  \n ").unwrap_err();

        assert!(rechazo.contains("diagram must not be empty."));
    }

    #[test]
    fn la_entrada_se_valida_antes_de_parsear() {
        let (mut flipchart, _commands) = pizarra();

        let rechazo = flipchart.show("", "esto no es Mermaid").unwrap_err();

        assert!(rechazo.contains("view_id must not be empty."));
    }

    #[test]
    fn un_rechazo_deja_intacta_la_vista_que_ya_habia() {
        let (mut flipchart, _commands) = pizarra();
        flipchart.show("actual", DOS_NODOS).unwrap();

        flipchart.show("actual", "").unwrap_err();

        assert_eq!(flipchart.view("actual"), Some(DOS_NODOS));
    }

    #[test]
    fn el_rechazo_abre_con_la_linea_fija() {
        let (mut flipchart, _commands) = pizarra();

        let rechazo = flipchart.show("actual", "").unwrap_err();

        assert!(
            rechazo.starts_with("Rejected: nothing was drawn; view \"actual\" is unchanged.\n")
        );
    }

    #[test]
    fn borrar_una_vista_la_dice_y_lista_lo_que_queda() {
        let (mut flipchart, _commands) = pizarra();
        flipchart.show("actual", DOS_NODOS).unwrap();
        flipchart.show("propuesto", DOS_NODOS).unwrap();

        let texto = flipchart.clear(Some("propuesto"));

        assert_eq!(
            texto,
            "Cleared view \"propuesto\". Views on the flipchart: actual."
        );
    }

    #[test]
    fn borrar_la_ultima_vista_deja_la_pizarra_sin_vistas() {
        let (mut flipchart, _commands) = pizarra();
        flipchart.show("actual", DOS_NODOS).unwrap();

        let texto = flipchart.clear(Some("actual"));

        assert_eq!(texto, "Cleared view \"actual\". No views.");
    }

    #[test]
    fn borrar_la_pizarra_entera_lo_dice() {
        let (mut flipchart, _commands) = pizarra();
        flipchart.show("actual", DOS_NODOS).unwrap();

        let texto = flipchart.clear(None);

        assert_eq!(texto, "Cleared the flipchart. No views.");
    }

    #[test]
    fn borrar_un_id_que_no_existe_no_es_error_y_lleva_la_lista_al_lado() {
        let (mut flipchart, _commands) = pizarra();
        flipchart.show("actual", DOS_NODOS).unwrap();
        flipchart.show("propuesto", DOS_NODOS).unwrap();

        let texto = flipchart.clear(Some("propeusto"));

        assert_eq!(
            texto,
            "No view \"propeusto\" on the flipchart. Views: actual, propuesto."
        );
    }

    #[test]
    fn borrar_un_id_que_no_existe_no_toca_las_vistas() {
        let (mut flipchart, _commands) = pizarra();
        flipchart.show("actual", DOS_NODOS).unwrap();

        flipchart.clear(Some("propeusto"));

        assert_eq!(flipchart.view("actual"), Some(DOS_NODOS));
    }

    #[test]
    fn borrar_la_pizarra_ya_vacia_lo_dice() {
        let (mut flipchart, _commands) = pizarra();

        let texto = flipchart.clear(None);

        assert_eq!(texto, "The flipchart was already empty.");
    }

    #[test]
    fn el_acuse_arrastra_los_avisos_detras() {
        let (mut flipchart, _commands) = pizarra();

        let acuse = flipchart.show("actual", TRES_NODOS_HACIA_ABAJO).unwrap();

        assert_eq!(
            acuse,
            "Shown as view \"actual\" (3 nodes, 2 edges). Views on the flipchart: actual.\n\
             Note: the flipchart lays diagrams out left to right; the direction in your \
             source was ignored. The view was drawn."
        );
    }

    #[test]
    fn un_rechazo_no_lleva_avisos() {
        let (mut flipchart, _commands) = pizarra();

        let rechazo = flipchart
            .show(
                "actual",
                "flowchart TB\n  classDef danger fill:#f00\n  A[Uno] --> Db\n",
            )
            .unwrap_err();

        assert!(!rechazo.contains("Note:"));
    }

    #[test]
    fn un_diagrama_que_no_parsea_se_rechaza_sin_tocar_la_pizarra() {
        let (mut flipchart, _commands) = pizarra();

        let rechazo = flipchart.show("actual", "esto no es Mermaid").unwrap_err();

        assert!(rechazo.starts_with("Rejected: nothing was drawn;"));
        assert_eq!(flipchart.view("actual"), None);
    }
}
