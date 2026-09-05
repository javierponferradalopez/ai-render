//! The SVG to pixels, on a worker thread: the event loop only uploads the texture.

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, TryIter, channel};
use std::thread;

use eframe::egui;
use resvg::{tiny_skia, usvg};

/// A scale rounded to eighths. Rasterising on every pixel of a drag would put
/// the whole SVG through `resvg` dozens of times a second.
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
    Deck { sheets: Vec<(u64, String)> },
    Paint { sheet: u64, scale: Scale },
}

/// Always stamped with the sheet it came from: a `show` during rasterisation
/// leaves whatever was on its way obsolete.
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

    /// The live sheets, in the order they are in: the ones the thread did not
    /// have are read and measured, and the ones no longer coming are dropped.
    pub fn deck(&self, sheets: Vec<(u64, String)>) {
        let _ = self.jobs.send(Job::Deck { sheets });
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
    let mut loaded: HashMap<u64, usvg::Tree> = HashMap::new();
    while let Ok(job) = jobs.recv() {
        let measured = match job {
            Job::Deck { sheets } => {
                loaded.retain(|sheet, _| sheets.iter().any(|(alive, _)| alive == sheet));
                read(&sheets, &mut loaded, &options)
            }
            Job::Paint { sheet, scale } => match loaded.get(&sheet) {
                Some(tree) => vec![Rendered::Painted {
                    sheet,
                    scale,
                    image: rasterize(tree, scale.factor()),
                }],
                None => continue,
            },
        };
        for rendered in measured {
            if done.send(rendered).is_err() {
                return;
            }
        }
        ctx.request_repaint();
    }
}

fn read(
    sheets: &[(u64, String)],
    loaded: &mut HashMap<u64, usvg::Tree>,
    options: &usvg::Options<'_>,
) -> Vec<Rendered> {
    let mut measured = Vec::new();
    for (sheet, svg) in sheets {
        if loaded.contains_key(sheet) {
            continue;
        }
        let Some(tree) = parse(svg, options) else {
            continue;
        };
        measured.push(Rendered::Measured {
            sheet: *sheet,
            natural: natural(&tree),
        });
        loaded.insert(*sheet, tree);
    }
    measured
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
            eprintln!("[flipchart] the drawn SVG cannot be read: {error}");
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

    let mut pixmap = tiny_skia::Pixmap::new(width, height).expect("a bounded side always fits");
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

    const SQUARE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"><rect width="100" height="50" fill="black"/></svg>"#;

    fn square() -> usvg::Tree {
        parse(SQUARE, &system_fonts()).expect("the test SVG reads")
    }

    #[test]
    fn the_natural_size_is_the_svgs() {
        assert_eq!(natural(&square()), egui::vec2(100.0, 50.0));
    }

    #[test]
    fn rasterising_at_natural_size_gives_the_svgs_pixels() {
        let image = rasterize(&square(), 1.0);

        assert_eq!(image.size, [100, 50]);
    }

    #[test]
    fn rasterising_at_twice_gives_twice_the_pixels() {
        let image = rasterize(&square(), 2.0);

        assert_eq!(image.size, [200, 100]);
    }

    #[test]
    fn an_svg_that_does_not_read_does_not_bring_the_thread_down() {
        assert!(parse("this is not an SVG", &system_fonts()).is_none());
    }

    #[test]
    fn a_sheet_already_read_is_not_read_again() {
        let options = system_fonts();
        let mut loaded = HashMap::new();
        read(&[(1, SQUARE.to_string())], &mut loaded, &options);

        let measured = read(
            &[(1, SQUARE.to_string()), (2, SQUARE.to_string())],
            &mut loaded,
            &options,
        );

        assert!(matches!(
            measured.as_slice(),
            [Rendered::Measured { sheet: 2, .. }]
        ));
    }

    #[test]
    fn the_scale_is_rounded_to_eighths() {
        assert_eq!(Scale::nearest(0.7).factor(), 0.75);
        assert_eq!(Scale::nearest(1.0).factor(), 1.0);
    }

    #[test]
    fn two_scales_of_the_same_eighth_are_the_same_and_reuse_the_texture() {
        assert_eq!(Scale::nearest(0.74), Scale::nearest(0.76));
    }

    #[test]
    fn the_scale_never_reaches_zero() {
        assert!(Scale::nearest(0.0).factor() > 0.0);
    }
}
