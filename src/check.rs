#![allow(
    clippy::print_stdout,
    reason = "el subcomando de diagnóstico es dueño de stdout: no hay sesión MCP que corromper"
)]

use std::path::Path;

use crate::diagram;

/// Corre la tubería sobre ficheros `.mmd` e imprime el desenlace y su texto, sin
/// abrir ventana. Es lo que hace medibles las reglas del Límite honesto.
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
    fn un_diagrama_que_se_dibuja_dice_el_desenlace_y_el_recuento() {
        let texto = outcome_of("flowchart LR\n  A[Uno] --> B[Dos]\n", "dos-nodos");

        assert_eq!(texto, "drawn\n2 nodes, 1 edge");
    }

    #[test]
    fn un_dibujo_con_aviso_lo_imprime_detras_del_recuento() {
        let texto = outcome_of("flowchart TB\n  A[Uno] --> B[Dos]\n", "dos-nodos");

        assert_eq!(
            texto,
            "drawn\n2 nodes, 1 edge\n\
             Note: the flipchart lays diagrams out left to right; the direction in your \
             source was ignored. The view was drawn."
        );
    }

    #[test]
    fn un_rechazo_dice_el_desenlace_y_el_texto_que_recibiria_el_agente() {
        let texto = outcome_of("flowchart TD\n  API[API Layer] --> Db\n", "fc-99");

        assert_eq!(
            texto,
            "undeclared nodes\n\
             Rejected: nothing was drawn; view \"fc-99\" is unchanged.\n\
             1 node appears in the drawing that you did not declare.\n  \
             \"Db\"  line 2  — only used in a relation\n\
             Declare every node you name, and rewrite any line the renderer turned into one."
        );
    }

    #[test]
    fn el_view_id_del_diagnostico_es_el_nombre_del_fichero() {
        assert_eq!(
            view_id("cases/fc-11-header-desnuda.mmd"),
            "fc-11-header-desnuda"
        );
    }
}
