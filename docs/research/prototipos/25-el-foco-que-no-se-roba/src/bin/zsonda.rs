//! La sonda desde fuera: el orden Z de las ventanas en pantalla, para medir el
//! **producto** —que no tiene comando de diagnóstico— igual que se midió el
//! spike.
//!
//! Uso: zsonda <pid> [nombre-del-terminal]
//! Escribe una línea de JSON: dónde está la ventana de ese pid, y si está por
//! encima de la del terminal.

use foco_spike::ventanas;

fn main() {
    let mut argumentos = std::env::args().skip(1);
    let pid: i32 = argumentos
        .next()
        .and_then(|v| v.parse().ok())
        .expect("uso: zsonda <pid> [nombre-del-terminal]");
    let terminal = argumentos.next().unwrap_or_else(|| "Alacritty".to_string());

    let ventanas = ventanas::de_delante_hacia_atras();
    let mia = ventanas.iter().position(|v| v.pid == pid);
    let suya = ventanas
        .iter()
        .position(|v| v.owner.eq_ignore_ascii_case(&terminal));
    let encima = match (mia, suya) {
        (Some(mia), Some(suya)) => Some(mia < suya),
        _ => None,
    };
    let en_pantalla: Vec<String> = ventanas
        .iter()
        .map(|v| format!("\"{}\"", v.owner.replace('"', "'")))
        .collect();
    println!(
        "{{\"mi_z\": {}, \"z_terminal\": {}, \"encima_del_terminal\": {}, \"delante\": {}, \"en_pantalla\": [{}]}}",
        opcional(mia),
        opcional(suya),
        encima.map(|e| e.to_string()).unwrap_or("null".into()),
        en_pantalla.first().map(String::as_str).unwrap_or("null"),
        en_pantalla.join(", "),
    );
}

fn opcional(valor: Option<usize>) -> String {
    valor.map(|v| v.to_string()).unwrap_or("null".into())
}
