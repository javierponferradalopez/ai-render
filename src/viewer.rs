//! El Visor: la ventana que enseña la hoja que el agente acaba de mostrar.

use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui;
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

use crate::mac::take_the_focus;
use crate::raster::{Rasterizer, Rendered, Scale};

#[derive(Debug)]
pub enum ViewerCommand {
    Show { view_id: String, svg: String },
    Clear,
}

/// El canal en memoria del Servidor al Visor. Es más que un `Sender`: despierta
/// al event loop, que macOS no ralentiza sino que **para** con la ventana
/// tapada — y tapada es el caso normal, con el usuario en su terminal.
#[derive(Debug)]
pub struct Wire {
    commands: Sender<ViewerCommand>,
    awake: Waker,
}

impl Wire {
    pub fn send(&self, command: ViewerCommand) {
        let _ = self.commands.send(command);
        if let Some(ctx) = self.awake.get() {
            ctx.request_repaint();
        }
    }
}

#[derive(Debug)]
pub struct Commands {
    commands: Receiver<ViewerCommand>,
    awake: Waker,
}

impl Commands {
    pub(crate) fn recv(&self) -> Option<ViewerCommand> {
        self.commands.recv().ok()
    }

    fn try_recv(&self) -> Option<ViewerCommand> {
        self.commands.try_recv().ok()
    }
}

type Waker = Arc<OnceLock<egui::Context>>;

pub fn wire() -> (Wire, Commands) {
    let (commands, pending) = channel();
    let awake: Waker = Arc::default();
    (
        Wire {
            commands,
            awake: awake.clone(),
        },
        Commands {
            commands: pending,
            awake,
        },
    )
}

/// Arranque diferido: el hilo principal se queda en el canal y no crea el event
/// loop —ni los 97 MB que cuesta— hasta que el agente pide el primer dibujo.
pub fn open_at_the_first_show(commands: Commands) {
    let Some(first) = wait_for_the_first_show(&commands) else {
        return;
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title())
            .with_inner_size([1200.0, 800.0]),
        // La subida de `Accessory` a `Regular` ocurre al construir el event loop,
        // que es exactamente el primer `show`. Tiene que ser aquí y no después:
        // medido, una app que nació accesoria no se activa nunca —ni cambiando
        // la política ni diez frames más tarde— y la ventana aparece detrás del
        // terminal mientras el agente dice que ha dibujado.
        event_loop_builder: Some(Box::new(|builder| {
            builder.with_activation_policy(ActivationPolicy::Regular);
        })),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "flipchart",
        options,
        Box::new(move |cc| Ok(Box::new(Viewer::new(cc, commands, first)))),
    );
}

fn wait_for_the_first_show(commands: &Commands) -> Option<ViewerCommand> {
    loop {
        match commands.recv()? {
            show @ ViewerCommand::Show { .. } => return Some(show),
            ViewerCommand::Clear => {}
        }
    }
}

fn title() -> String {
    match working_directory() {
        Some(directory) => format!("Pizarra — {directory}"),
        None => "Pizarra".to_string(),
    }
}

fn working_directory() -> Option<String> {
    let path = std::env::current_dir().ok()?;
    Some(path.file_name()?.to_string_lossy().into_owned())
}

struct Sheet {
    number: u64,
    id: String,
    natural: Option<egui::Vec2>,
    painted: Option<(Scale, egui::TextureHandle)>,
    awaited: Option<Scale>,
}

impl Sheet {
    fn new(number: u64, id: String) -> Self {
        Self {
            number,
            id,
            natural: None,
            painted: None,
            awaited: None,
        }
    }
}

struct Viewer {
    commands: Commands,
    rasterizer: Rasterizer,
    sheet: Option<Sheet>,
    shown: u64,
    focus: Focus,
}

impl Viewer {
    fn new(cc: &eframe::CreationContext<'_>, commands: Commands, first: ViewerCommand) -> Self {
        let _ = commands.awake.set(cc.egui_ctx.clone());
        let mut viewer = Self {
            commands,
            rasterizer: Rasterizer::spawn(cc.egui_ctx.clone()),
            sheet: None,
            shown: 0,
            focus: Focus::default(),
        };
        viewer.accept(first);
        viewer
    }

    fn accept(&mut self, command: ViewerCommand) {
        match command {
            ViewerCommand::Show { view_id, svg } => {
                self.shown += 1;
                self.sheet = Some(Sheet::new(self.shown, view_id));
                self.rasterizer.load(self.shown, svg);
            }
            ViewerCommand::Clear => self.sheet = None,
        }
    }

    fn collect(&mut self, ctx: &egui::Context) {
        for rendered in self.rasterizer.collect() {
            let Some(sheet) = self.sheet.as_mut() else {
                continue;
            };
            match rendered {
                Rendered::Measured {
                    sheet: number,
                    natural,
                } if number == sheet.number => {
                    sheet.natural = Some(natural);
                }
                Rendered::Painted {
                    sheet: number,
                    scale,
                    image,
                } if number == sheet.number => {
                    let texture = ctx.load_texture(
                        format!("{}#{number}", sheet.id),
                        image,
                        egui::TextureOptions::LINEAR,
                    );
                    sheet.awaited = None;
                    sheet.painted = Some((scale, texture));
                }
                _ => {}
            }
        }
    }

    fn request(
        &mut self,
        room: egui::Vec2,
        points_per_pixel: f32,
    ) -> Option<(egui::TextureHandle, egui::Vec2)> {
        let sheet = self.sheet.as_mut()?;
        let natural = sheet.natural?;

        let zoom = fit(natural, room);
        let wanted = Scale::nearest(zoom * points_per_pixel);
        let up_to_date = sheet
            .painted
            .as_ref()
            .is_some_and(|(scale, _)| *scale == wanted);
        if !up_to_date && sheet.awaited != Some(wanted) {
            sheet.awaited = Some(wanted);
            self.rasterizer.paint(sheet.number, wanted);
        }

        let (_, texture) = sheet.painted.as_ref()?;
        Some((texture.clone(), natural * zoom))
    }
}

const MINIMUM_ZOOM: f32 = 0.05;

/// El foco se roba cuando la ventana nace y **nunca en una actualización**:
/// saltar al frente cada vez que el agente retoca, mientras el usuario escribe
/// en la terminal, es intolerable.
#[derive(Debug, Default)]
struct Focus {
    stolen: bool,
}

impl Focus {
    fn steal(&mut self) -> bool {
        !std::mem::replace(&mut self.stolen, true)
    }
}

/// Encaje: encoger, nunca agrandar. Agrandar miente — pone un diagrama de tres
/// nodos al 128 % al lado de uno de veinte al 27 %.
fn fit(natural: egui::Vec2, room: egui::Vec2) -> f32 {
    if natural.x <= 0.0 || natural.y <= 0.0 {
        return 1.0;
    }
    (room.x / natural.x)
        .min(room.y / natural.y)
        .clamp(MINIMUM_ZOOM, 1.0)
}

impl eframe::App for Viewer {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.focus.steal() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
            take_the_focus();
        }

        while let Some(command) = self.commands.try_recv() {
            self.accept(command);
        }
        let ctx = ui.ctx().clone();
        self.collect(&ctx);

        let Some(id) = self.sheet.as_ref().map(|sheet| sheet.id.clone()) else {
            ui.weak("La pizarra está vacía.");
            return;
        };

        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            ui.strong(id);
            ui.add_space(6.0);

            let room = ui.available_size();
            if let Some((texture, size)) = self.request(room, ctx.pixels_per_point()) {
                ui.add(egui::Image::new(&texture).fit_to_exact_size(size));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_foco_se_roba_una_sola_vez() {
        let mut focus = Focus::default();

        assert!(focus.steal());
        assert!(!focus.steal());
        assert!(!focus.steal());
    }

    #[test]
    fn una_hoja_que_no_cabe_se_encoge_hasta_caber() {
        let zoom = fit(egui::vec2(2400.0, 800.0), egui::vec2(1200.0, 800.0));

        assert_eq!(zoom, 0.5);
    }

    #[test]
    fn una_hoja_que_cabe_se_deja_a_su_tamano_y_no_se_agranda() {
        let zoom = fit(egui::vec2(300.0, 200.0), egui::vec2(1200.0, 800.0));

        assert_eq!(zoom, 1.0);
    }

    #[test]
    fn manda_el_lado_que_peor_cabe() {
        let zoom = fit(egui::vec2(1200.0, 3200.0), egui::vec2(1200.0, 800.0));

        assert_eq!(zoom, 0.25);
    }

    #[test]
    fn una_ventana_encogida_a_nada_no_pide_una_escala_de_cero() {
        let zoom = fit(egui::vec2(1200.0, 800.0), egui::vec2(1.0, 1.0));

        assert_eq!(zoom, MINIMUM_ZOOM);
    }

    #[test]
    fn el_titulo_lleva_el_directorio_de_trabajo() {
        let esperado = format!("Pizarra — {}", working_directory().unwrap());

        assert_eq!(title(), esperado);
    }

    #[test]
    fn el_primer_show_es_el_que_saca_la_ventana_y_un_clear_solo_no_la_saca() {
        let (viewer, commands) = wire();
        viewer.send(ViewerCommand::Clear);
        viewer.send(ViewerCommand::Show {
            view_id: "actual".to_string(),
            svg: "<svg/>".to_string(),
        });

        let primero = wait_for_the_first_show(&commands).unwrap();

        assert!(matches!(primero, ViewerCommand::Show { .. }));
    }

    #[test]
    fn una_sesion_que_muere_sin_dibujar_nada_no_abre_ventana() {
        let (viewer, commands) = wire();
        drop(viewer);

        assert!(wait_for_the_first_show(&commands).is_none());
    }
}
