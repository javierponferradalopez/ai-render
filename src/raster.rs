//! El SVG a píxeles, en un hilo de trabajo: el event loop sólo sube la textura.

use std::sync::mpsc::{Receiver, Sender, TryIter, channel};
use std::thread;

use eframe::egui;
use resvg::{tiny_skia, usvg};

/// Una escala redondeada a octavos. Rasterizar en cada píxel de arrastre pasaría
/// el SVG entero por `resvg` decenas de veces por segundo.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Scale(u32);

impl Scale {
    pub fn nearest(scale: f32) -> Self {
        Self(((scale * 8.0).round() as u32).max(1))
    }

    pub fn factor(self) -> f32 {
        self.0 as f32 / 8.0
    }
}

#[derive(Debug)]
enum Job {
    Load { sheet: u64, svg: String },
    Paint { sheet: u64, scale: Scale },
}

/// Marcado siempre con la hoja de la que salió: un `show` durante la
/// rasterización deja obsoleto lo que venía en camino.
#[derive(Debug)]
pub enum Rendered {
    Measured {
        sheet: u64,
        natural: egui::Vec2,
    },
    Painted {
        sheet: u64,
        scale: Scale,
        image: egui::ColorImage,
    },
}

#[derive(Debug)]
pub struct Rasterizer {
    jobs: Sender<Job>,
    rendered: Receiver<Rendered>,
}

impl Rasterizer {
    pub fn spawn(ctx: egui::Context) -> Self {
        let (jobs, pending) = channel();
        let (done, rendered) = channel();
        thread::spawn(move || work(&pending, &done, &ctx));
        Self { jobs, rendered }
    }

    pub fn load(&self, sheet: u64, svg: String) {
        let _ = self.jobs.send(Job::Load { sheet, svg });
    }

    pub fn paint(&self, sheet: u64, scale: Scale) {
        let _ = self.jobs.send(Job::Paint { sheet, scale });
    }

    pub fn collect(&self) -> TryIter<'_, Rendered> {
        self.rendered.try_iter()
    }
}

fn work(jobs: &Receiver<Job>, done: &Sender<Rendered>, ctx: &egui::Context) {
    let options = system_fonts();
    let mut loaded: Option<(u64, usvg::Tree)> = None;
    while let Ok(job) = jobs.recv() {
        let rendered = match job {
            Job::Load { sheet, svg } => match parse(&svg, &options) {
                Some(tree) => {
                    let natural = natural(&tree);
                    loaded = Some((sheet, tree));
                    Rendered::Measured { sheet, natural }
                }
                None => continue,
            },
            Job::Paint { sheet, scale } => match &loaded {
                Some((cargada, tree)) if *cargada == sheet => Rendered::Painted {
                    sheet,
                    scale,
                    image: rasterize(tree, scale.factor()),
                },
                _ => continue,
            },
        };
        if done.send(rendered).is_err() {
            return;
        }
        ctx.request_repaint();
    }
}

fn system_fonts() -> usvg::Options<'static> {
    let mut options = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    options
}

fn parse(svg: &str, options: &usvg::Options<'_>) -> Option<usvg::Tree> {
    match usvg::Tree::from_str(svg, options) {
        Ok(tree) => Some(tree),
        Err(error) => {
            eprintln!("[flipchart] el SVG dibujado no se puede leer: {error}");
            None
        }
    }
}

fn natural(tree: &usvg::Tree) -> egui::Vec2 {
    egui::vec2(tree.size().width(), tree.size().height())
}

const MAX_TEXTURE_SIDE: u32 = 8192;

fn rasterize(tree: &usvg::Tree, scale: f32) -> egui::ColorImage {
    let size = natural(tree);
    let width = ((size.x * scale).ceil() as u32).clamp(1, MAX_TEXTURE_SIDE);
    let height = ((size.y * scale).ceil() as u32).clamp(1, MAX_TEXTURE_SIDE);

    let mut pixmap = tiny_skia::Pixmap::new(width, height).expect("un lado acotado cabe siempre");
    pixmap.fill(tiny_skia::Color::WHITE);
    resvg::render(
        tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], pixmap.data())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUADRADO: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"><rect width="100" height="50" fill="black"/></svg>"#;

    fn cuadrado() -> usvg::Tree {
        parse(CUADRADO, &system_fonts()).expect("el SVG de prueba se lee")
    }

    #[test]
    fn el_natural_es_el_del_svg() {
        assert_eq!(natural(&cuadrado()), egui::vec2(100.0, 50.0));
    }

    #[test]
    fn rasterizar_al_natural_da_los_pixeles_del_svg() {
        let imagen = rasterize(&cuadrado(), 1.0);

        assert_eq!(imagen.size, [100, 50]);
    }

    #[test]
    fn rasterizar_a_dos_veces_da_el_doble_de_pixeles() {
        let imagen = rasterize(&cuadrado(), 2.0);

        assert_eq!(imagen.size, [200, 100]);
    }

    #[test]
    fn un_svg_que_no_se_lee_no_tumba_el_hilo() {
        assert!(parse("esto no es un SVG", &system_fonts()).is_none());
    }

    #[test]
    fn la_escala_se_redondea_a_octavos() {
        assert_eq!(Scale::nearest(0.7).factor(), 0.75);
        assert_eq!(Scale::nearest(1.0).factor(), 1.0);
    }

    #[test]
    fn dos_escalas_del_mismo_octavo_son_la_misma_y_reusan_la_textura() {
        assert_eq!(Scale::nearest(0.74), Scale::nearest(0.76));
    }

    #[test]
    fn la_escala_nunca_llega_a_cero() {
        assert!(Scale::nearest(0.0).factor() > 0.0);
    }
}
