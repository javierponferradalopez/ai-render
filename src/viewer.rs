//! El Visor: el rotafolio que enseña la hoja que el agente puso delante.

use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui;
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

use crate::mac::take_the_focus;
use crate::raster::{Rasterizer, Rendered, Scale};

/// Una hoja ya dibujada, tal como cruza. La numera quien la dibuja: un `show`
/// sobre un id que ya existía da una hoja nueva con el mismo nombre.
#[derive(Debug)]
pub struct Drawn {
    pub number: u64,
    pub id: String,
    pub svg: String,
}

/// La pizarra entera cada vez: las hojas **en orden de creación** y **cuál va
/// delante**. El Visor no decide ninguna de las dos cosas.
#[derive(Debug)]
pub struct DeckSnapshot {
    pub sheets: Vec<Drawn>,
    pub front: Option<usize>,
}

/// El canal en memoria del Servidor al Visor. Es más que un `Sender`: despierta
/// al event loop, que macOS no ralentiza sino que **para** con la ventana
/// tapada — y tapada es el caso normal, con el usuario en su terminal.
#[derive(Debug)]
pub struct Wire {
    commands: Sender<DeckSnapshot>,
    awake: Waker,
}

impl Wire {
    pub fn send(&self, snapshot: DeckSnapshot) {
        let _ = self.commands.send(snapshot);
        if let Some(ctx) = self.awake.get() {
            ctx.request_repaint();
        }
    }
}

#[derive(Debug)]
pub struct Commands {
    commands: Receiver<DeckSnapshot>,
    awake: Waker,
}

impl Commands {
    pub(crate) fn recv(&self) -> Option<DeckSnapshot> {
        self.commands.recv().ok()
    }

    pub(crate) fn try_recv(&self) -> Option<DeckSnapshot> {
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

fn wait_for_the_first_show(commands: &Commands) -> Option<DeckSnapshot> {
    loop {
        let snapshot = commands.recv()?;
        if !snapshot.sheets.is_empty() {
            return Some(snapshot);
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

/// Una hoja en pantalla: lo que el Visor recuerda de ella es su dibujo, y el
/// **zoom es suyo** — cada Vista guarda el que le tocó por su tamaño.
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

/// Las hojas que hay y cuál se mira. El cursor es **local del Visor** —el único
/// mando del usuario— y **lo pisa el siguiente `show`**: un `clear` lo deja
/// mirando donde estaba.
#[derive(Default)]
struct Deck {
    sheets: Vec<Sheet>,
    cursor: usize,
}

impl Deck {
    fn accept(&mut self, snapshot: &DeckSnapshot) {
        let looked_at = self.showing().map(|sheet| sheet.number);
        let front = snapshot.front.unwrap_or(0);
        let turned = snapshot
            .sheets
            .get(front)
            .is_some_and(|drawn| self.position(drawn.number).is_none());

        let mut previous = std::mem::take(&mut self.sheets);
        for drawn in &snapshot.sheets {
            let sheet = match previous.iter().position(|kept| kept.number == drawn.number) {
                Some(position) => previous.remove(position),
                None => Sheet::new(drawn.number, drawn.id.clone()),
            };
            self.sheets.push(sheet);
        }

        self.cursor = match looked_at {
            Some(number) if !turned => self.position(number).unwrap_or(front),
            _ => front,
        };
    }

    fn position(&self, number: u64) -> Option<usize> {
        self.sheets.iter().position(|sheet| sheet.number == number)
    }

    fn showing(&self) -> Option<&Sheet> {
        self.sheets.get(self.cursor)
    }

    fn showing_mut(&mut self) -> Option<&mut Sheet> {
        self.sheets.get_mut(self.cursor)
    }

    fn sheet_mut(&mut self, number: u64) -> Option<&mut Sheet> {
        self.sheets.iter_mut().find(|sheet| sheet.number == number)
    }

    fn back(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn forward(&mut self) {
        self.cursor = (self.cursor + 1).min(self.sheets.len().saturating_sub(1));
    }
}

struct Viewer {
    commands: Commands,
    rasterizer: Rasterizer,
    deck: Deck,
    focus: Focus,
}

impl Viewer {
    fn new(cc: &eframe::CreationContext<'_>, commands: Commands, first: DeckSnapshot) -> Self {
        let _ = commands.awake.set(cc.egui_ctx.clone());
        let mut viewer = Self {
            commands,
            rasterizer: Rasterizer::spawn(cc.egui_ctx.clone()),
            deck: Deck::default(),
            focus: Focus::default(),
        };
        viewer.accept(first);
        viewer
    }

    fn accept(&mut self, snapshot: DeckSnapshot) {
        self.deck.accept(&snapshot);
        self.rasterizer.deck(
            snapshot
                .sheets
                .into_iter()
                .map(|drawn| (drawn.number, drawn.svg))
                .collect(),
        );
    }

    fn collect(&mut self, ctx: &egui::Context) {
        for rendered in self.rasterizer.collect() {
            match rendered {
                Rendered::Measured {
                    sheet: number,
                    natural,
                } => {
                    if let Some(sheet) = self.deck.sheet_mut(number) {
                        sheet.natural = Some(natural);
                    }
                }
                Rendered::Painted {
                    sheet: number,
                    scale,
                    image,
                } => {
                    let Some(sheet) = self.deck.sheet_mut(number) else {
                        continue;
                    };
                    let name = format!("{}#{number}", sheet.id);
                    let texture = ctx.load_texture(name, image, egui::TextureOptions::LINEAR);
                    sheet.awaited = None;
                    sheet.painted = Some((scale, texture));
                }
            }
        }
    }

    fn request(
        &mut self,
        room: egui::Vec2,
        points_per_pixel: f32,
    ) -> Option<(egui::TextureHandle, egui::Vec2)> {
        let sheet = self.deck.showing_mut()?;
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

    /// La cabecera del rotafolio: la hoja, su nombre, dos flechas y «hoja N de
    /// M». **Sin índice** — un índice es mando de administración, y el usuario
    /// no administra: observa.
    fn header(&mut self, ui: &mut egui::Ui) {
        let sheets = self.deck.sheets.len();
        let cursor = self.deck.cursor;
        let Some(name) = self.deck.showing().map(|sheet| sheet.id.clone()) else {
            return;
        };
        ui.horizontal(|ui| {
            if ui.add_enabled(cursor > 0, egui::Button::new("‹")).clicked() {
                self.deck.back();
            }
            ui.strong(name);
            if ui
                .add_enabled(cursor + 1 < sheets, egui::Button::new("›"))
                .clicked()
            {
                self.deck.forward();
            }
            ui.weak(format!("hoja {} de {sheets}", cursor + 1));
        });
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

        while let Some(snapshot) = self.commands.try_recv() {
            self.accept(snapshot);
        }
        let ctx = ui.ctx().clone();
        self.collect(&ctx);

        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            if self.deck.sheets.is_empty() {
                ui.weak("La pizarra está vacía.");
                return;
            }
            self.header(ui);
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
        viewer.send(pizarra(vec![], None));
        viewer.send(pizarra(vec![dibujada(1, "actual")], Some(0)));

        let primera = wait_for_the_first_show(&commands).unwrap();

        assert_eq!(primera.sheets.len(), 1);
    }

    #[test]
    fn una_sesion_que_muere_sin_dibujar_nada_no_abre_ventana() {
        let (viewer, commands) = wire();
        drop(viewer);

        assert!(wait_for_the_first_show(&commands).is_none());
    }

    fn dibujada(number: u64, id: &str) -> Drawn {
        Drawn {
            number,
            id: id.to_string(),
            svg: "<svg/>".to_string(),
        }
    }

    fn pizarra(sheets: Vec<Drawn>, front: Option<usize>) -> DeckSnapshot {
        DeckSnapshot { sheets, front }
    }

    fn tres_hojas() -> Deck {
        let mut deck = Deck::default();
        deck.accept(&pizarra(
            vec![
                dibujada(1, "actual"),
                dibujada(2, "variante A"),
                dibujada(3, "variante B"),
            ],
            Some(2),
        ));
        deck
    }

    fn mirando(deck: &Deck) -> &str {
        &deck.showing().expect("hay una hoja delante").id
    }

    #[test]
    fn el_show_deja_su_hoja_delante() {
        let deck = tres_hojas();

        assert_eq!(mirando(&deck), "variante B");
    }

    #[test]
    fn la_flecha_de_atras_retrocede_una_hoja() {
        let mut deck = tres_hojas();

        deck.back();

        assert_eq!(mirando(&deck), "variante A");
    }

    #[test]
    fn la_flecha_de_adelante_vuelve_a_la_hoja_siguiente() {
        let mut deck = tres_hojas();
        deck.back();

        deck.forward();

        assert_eq!(mirando(&deck), "variante B");
    }

    #[test]
    fn la_primera_hoja_no_tiene_anterior() {
        let mut deck = tres_hojas();

        deck.back();
        deck.back();
        deck.back();

        assert_eq!(mirando(&deck), "actual");
    }

    #[test]
    fn la_ultima_hoja_no_tiene_siguiente() {
        let mut deck = tres_hojas();

        deck.forward();

        assert_eq!(mirando(&deck), "variante B");
    }

    #[test]
    fn el_siguiente_show_pisa_el_cursor_del_usuario() {
        let mut deck = tres_hojas();
        deck.back();
        deck.back();

        deck.accept(&pizarra(
            vec![
                dibujada(1, "actual"),
                dibujada(2, "variante A"),
                dibujada(3, "variante B"),
                dibujada(4, "variante C"),
            ],
            Some(3),
        ));

        assert_eq!(mirando(&deck), "variante C");
    }

    #[test]
    fn retirar_una_hoja_que_no_se_miraba_deja_al_usuario_donde_estaba() {
        let mut deck = tres_hojas();
        deck.back();

        deck.accept(&pizarra(
            vec![dibujada(2, "variante A"), dibujada(3, "variante B")],
            Some(1),
        ));

        assert_eq!(mirando(&deck), "variante A");
    }

    #[test]
    fn retirar_la_hoja_que_se_miraba_pasa_a_la_que_va_delante() {
        let mut deck = tres_hojas();
        deck.back();

        deck.accept(&pizarra(
            vec![dibujada(1, "actual"), dibujada(3, "variante B")],
            Some(1),
        ));

        assert_eq!(mirando(&deck), "variante B");
    }

    #[test]
    fn una_hoja_viva_conserva_lo_que_ya_tenia_dibujado() {
        let mut deck = tres_hojas();
        deck.sheets[0].natural = Some(egui::vec2(300.0, 200.0));

        deck.accept(&pizarra(
            vec![
                dibujada(1, "actual"),
                dibujada(2, "variante A"),
                dibujada(3, "variante B"),
                dibujada(4, "variante C"),
            ],
            Some(3),
        ));

        assert_eq!(deck.sheets[0].natural, Some(egui::vec2(300.0, 200.0)));
    }

    #[test]
    fn reemplazar_una_vista_da_una_hoja_nueva_que_se_vuelve_a_dibujar() {
        let mut deck = tres_hojas();
        deck.sheets[0].natural = Some(egui::vec2(300.0, 200.0));

        deck.accept(&pizarra(
            vec![
                dibujada(4, "actual"),
                dibujada(2, "variante A"),
                dibujada(3, "variante B"),
            ],
            Some(0),
        ));

        assert_eq!(deck.sheets[0].natural, None);
    }

    #[test]
    fn la_pizarra_vaciada_se_queda_sin_hojas_que_ensenar() {
        let mut deck = tres_hojas();

        deck.accept(&pizarra(vec![], None));

        assert!(deck.showing().is_none());
    }
}
