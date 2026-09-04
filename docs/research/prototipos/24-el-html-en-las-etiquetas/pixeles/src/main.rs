//! Los SVG del banco a PNG, por el mismo camino que la ventana.
//!
//! `src/raster.rs` del producto hace exactamente esto —`usvg` con las fuentes
//! del sistema y `resvg` sobre un `tiny_skia::Pixmap`—, así que lo que sale
//! aquí son los píxeles que el Visor sube a la textura. Existe porque
//! `screencapture` necesita permiso de grabación de pantalla y la ventana no
//! siempre se puede fotografiar; `mira.py` es el camino cuando sí.

use resvg::{tiny_skia, usvg};
use std::path::PathBuf;

const ESCALA: f32 = 2.0;

fn main() {
    let mut opciones = usvg::Options::default();
    opciones.fontdb_mut().load_system_fonts();

    for ruta in std::env::args().skip(1).map(PathBuf::from) {
        let fuente = std::fs::read_to_string(&ruta).expect("el SVG del banco se lee");
        let arbol = usvg::Tree::from_str(&fuente, &opciones).expect("el SVG dibujado se parsea");

        let tamano = arbol.size().to_int_size().scale_by(ESCALA).unwrap();
        let mut mapa = tiny_skia::Pixmap::new(tamano.width(), tamano.height()).unwrap();
        mapa.fill(tiny_skia::Color::WHITE);
        resvg::render(
            &arbol,
            tiny_skia::Transform::from_scale(ESCALA, ESCALA),
            &mut mapa.as_mut(),
        );

        let destino = ruta.with_extension("png");
        mapa.save_png(&destino).expect("el PNG se escribe");
        eprintln!("{}", destino.display());
    }
}
