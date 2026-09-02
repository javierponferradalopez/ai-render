//! Maqueta para "Cómo se ven N vistas en una ventana" (#19).
//!
//! No es el producto: es una ventana `egui` con SVGs reales de mmdr dentro,
//! y todas las respuestas candidatas del ticket puestas como interruptores
//! para poder mirarlas en vez de discutirlas.
//!
//! Uso:
//!   n-vistas                       ventana interactiva
//!   n-vistas --captura <dir>       pinta cada disposición y guarda un PNG

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eframe::egui;
use resvg::tiny_skia;
use resvg::usvg;

// ---------------------------------------------------------------- disposición

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Disposicion {
    /// Una vista a la vez, a pantalla completa.
    Pestanas,
    /// Todas a la vez, una al lado de otra, en una sola fila.
    Columnas,
    /// Todas a la vez, cada una en su celda.
    Rejilla,
    /// Apiladas en vertical, scroll de la ventana.
    Apiladas,
    /// Apiladas, pero una maximizada al hacer clic en su nombre.
    Mezcla,
}

impl Disposicion {
    const TODAS: [Disposicion; 5] = [
        Disposicion::Pestanas,
        Disposicion::Columnas,
        Disposicion::Rejilla,
        Disposicion::Apiladas,
        Disposicion::Mezcla,
    ];
    fn nombre(self) -> &'static str {
        match self {
            Disposicion::Pestanas => "pestañas",
            Disposicion::Columnas => "columnas",
            Disposicion::Rejilla => "rejilla",
            Disposicion::Apiladas => "apiladas",
            Disposicion::Mezcla => "mezcla",
        }
    }
}

/// Cómo se traduce el tamaño natural de una Vista al hueco que le toca.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Encaje {
    /// 1:1. Ninguna se encoge; caben las que caben.
    Natural,
    /// Se encoge la que no cabe de ancho; ninguna se agranda.
    SinAgrandar,
    /// Cada una llena su hueco: las pequeñas se agrandan, las grandes se encogen.
    AlHueco,
    /// Una sola escala para todas, la que hace caber a la más grande.
    Comun,
}

impl Encaje {
    const TODOS: [Encaje; 4] = [
        Encaje::Natural,
        Encaje::SinAgrandar,
        Encaje::AlHueco,
        Encaje::Comun,
    ];
    fn nombre(self) -> &'static str {
        match self {
            Encaje::Natural => "natural (1:1)",
            Encaje::SinAgrandar => "encoger, nunca agrandar",
            Encaje::AlHueco => "cada una a su hueco",
            Encaje::Comun => "escala común",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Orden {
    Creacion,
    Alfabetico,
    UltimoShow,
}

impl Orden {
    const TODOS: [Orden; 3] = [Orden::Creacion, Orden::Alfabetico, Orden::UltimoShow];
    fn nombre(self) -> &'static str {
        match self {
            Orden::Creacion => "creación",
            Orden::Alfabetico => "alfabético",
            Orden::UltimoShow => "último show",
        }
    }
}

/// Qué hace la ventana cuando un `show` cae sobre una Vista que ya existe.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AlReemplazar {
    /// La ventana no se mueve. El usuario puede no enterarse.
    Quieta,
    /// El scroll salta a la Vista reemplazada.
    Salta,
    /// La ventana no se mueve, pero la Vista se marca unos segundos.
    QuietaConMarca,
}

impl AlReemplazar {
    const TODOS: [AlReemplazar; 3] = [
        AlReemplazar::Quieta,
        AlReemplazar::Salta,
        AlReemplazar::QuietaConMarca,
    ];
    fn nombre(self) -> &'static str {
        match self {
            AlReemplazar::Quieta => "quieta",
            AlReemplazar::Salta => "salta a ella",
            AlReemplazar::QuietaConMarca => "quieta + marca",
        }
    }
}

/// Cómo desaparece una Vista suelta de la Pizarra. Hoy no desaparece: `show`
/// la reemplaza y `clear` las mata todas.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Cierre {
    /// El usuario la cierra con la ✕ de su pestaña.
    Usuario,
    /// El agente la retira con una tercera herramienta.
    Agente,
    /// Caen solas al pasar de N, sin que nadie lo pida.
    Tope,
}

impl Cierre {
    const TODOS: [Cierre; 3] = [Cierre::Usuario, Cierre::Agente, Cierre::Tope];
    fn nombre(self) -> &'static str {
        match self {
            Cierre::Usuario => "la cierras tú",
            Cierre::Agente => "la quita el agente",
            Cierre::Tope => "caen solas pasadas 4",
        }
    }
}

/// Tope de Vistas vivas en el escenario `Cierre::Tope`.
const TOPE: usize = 4;

// --------------------------------------------------------------------- vistas

/// Una Vista de la Pizarra: su id, su SVG ya parseado y su tamaño natural.
struct Vista {
    id: String,
    arbol: usvg::Tree,
    natural: egui::Vec2,
    /// zoom propio, sólo se usa cuando el zoom no es de la ventana entera
    zoom: f32,
    /// sube en cada reemplazo; invalida la textura y dispara la marca
    generacion: u32,
    /// segundos de vida que le quedan a la marca de reemplazo
    marca: f32,
    /// contador del último `show` que la tocó, para el orden
    ultimo_show: u64,
    /// apagarla simula una Pizarra con menos Vistas vivas
    visible: bool,
}

fn carga_svg(ruta: &Path) -> usvg::Tree {
    let datos = std::fs::read(ruta).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()));
    let mut opciones = usvg::Options::default();
    opciones.fontdb_mut().load_system_fonts();
    usvg::Tree::from_data(&datos, &opciones).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()))
}

fn tamano(arbol: &usvg::Tree) -> egui::Vec2 {
    egui::vec2(arbol.size().width(), arbol.size().height())
}

/// Rasteriza el SVG a la escala pedida. Es lo que hará el producto: al Visor
/// llega el SVG y `resvg` lo pasa a píxeles al zoom y al DPI del momento.
fn rasteriza(arbol: &usvg::Tree, escala: f32) -> egui::ColorImage {
    let natural = tamano(arbol);
    let w = ((natural.x * escala).ceil() as u32).clamp(1, 8192);
    let h = ((natural.y * escala).ceil() as u32).clamp(1, 8192);
    let mut pixmap = tiny_skia::Pixmap::new(w, h).expect("pixmap");
    pixmap.fill(tiny_skia::Color::WHITE);
    resvg::render(
        arbol,
        tiny_skia::Transform::from_scale(escala, escala),
        &mut pixmap.as_mut(),
    );
    // el fondo es opaco, así que premultiplicado y sin premultiplicar coinciden
    egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], pixmap.data())
}

// ------------------------------------------------------------------------ app

struct Maqueta {
    vistas: Vec<Vista>,
    /// SVG de recambio, para simular un `show` sobre un id que ya existe
    recambio: Option<usvg::Tree>,
    texturas: HashMap<(String, u32, u32), egui::TextureHandle>,

    disposicion: Disposicion,
    encaje: Encaje,
    orden: Orden,
    al_reemplazar: AlReemplazar,
    zoom_compartido: bool,
    zoom_ventana: f32,
    mostrar_nombres: bool,

    /// pestaña activa / vista maximizada en el modo mezcla
    activa: usize,
    maximizada: Option<usize>,
    contador_show: u64,
    /// vista a la que hay que llevar el scroll en este frame
    ir_a: Option<usize>,

    captura: Option<PathBuf>,
    /// ids que sobreviven a la segunda ronda de capturas
    pares: Vec<String>,
    cierre: Cierre,
    /// retratar el guion de escenarios en vez de las disposiciones
    guion: bool,
    /// esconde los interruptores, para ver la ventana como sería de verdad
    limpio: bool,
    /// cuántas Vistas se han ido ya, para el rastro del escenario del tope
    retiradas: usize,
    frames: u64,
}

impl Maqueta {
    fn nueva(dir: &Path, captura: Option<PathBuf>) -> Self {
        let mut ficheros: Vec<PathBuf> = std::fs::read_dir(dir)
            .expect("falta vistas/ — corre ./genera-vistas.sh")
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().map(|e| e == "svg").unwrap_or(false))
            .collect();
        ficheros.sort();

        let mut vistas = Vec::new();
        let mut recambio = None;
        for (i, f) in ficheros.iter().enumerate() {
            let tallo = f.file_stem().unwrap().to_string_lossy().to_string();
            if tallo.starts_with('x') {
                recambio = Some(carga_svg(f));
                continue;
            }
            // el prefijo numérico sólo ordena los ficheros; el id es lo que el
            // agente teclearía
            let id = tallo.splitn(2, '-').nth(1).unwrap_or(&tallo).to_string();
            let arbol = carga_svg(f);
            let natural = tamano(&arbol);
            vistas.push(Vista {
                id,
                arbol,
                natural,
                zoom: 1.0,
                generacion: 0,
                marca: 0.0,
                ultimo_show: i as u64,
                visible: true,
            });
        }
        assert!(!vistas.is_empty(), "no hay SVGs en {}", dir.display());

        Self {
            contador_show: vistas.len() as u64,
            vistas,
            recambio,
            texturas: HashMap::new(),
            disposicion: Disposicion::Apiladas,
            encaje: Encaje::SinAgrandar,
            orden: Orden::Creacion,
            al_reemplazar: AlReemplazar::QuietaConMarca,
            zoom_compartido: true,
            zoom_ventana: 1.0,
            mostrar_nombres: true,
            activa: 0,
            maximizada: None,
            ir_a: None,
            captura,
            pares: Vec::new(),
            cierre: Cierre::Usuario,
            guion: false,
            limpio: false,
            retiradas: 0,
            frames: 0,
        }
    }

    /// El orden en que se recorren las vistas, como índices.
    fn indices(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.vistas.len())
            .filter(|i| self.vistas[*i].visible)
            .collect();
        match self.orden {
            Orden::Creacion => {}
            Orden::Alfabetico => idx.sort_by(|a, b| self.vistas[*a].id.cmp(&self.vistas[*b].id)),
            // el último `show` primero: lo recién dicho arriba
            Orden::UltimoShow => {
                idx.sort_by(|a, b| self.vistas[*b].ultimo_show.cmp(&self.vistas[*a].ultimo_show))
            }
        }
        idx
    }

    fn zoom_de(&self, i: usize) -> f32 {
        if self.zoom_compartido {
            self.zoom_ventana
        } else {
            self.vistas[i].zoom
        }
    }

    /// Escala efectiva de una vista dentro del hueco que le toca.
    fn escala(&self, i: usize, hueco: egui::Vec2) -> f32 {
        let ajuste = |n: egui::Vec2| (hueco.x / n.x).min(hueco.y / n.y);
        let base = match self.encaje {
            Encaje::Natural => 1.0,
            Encaje::SinAgrandar => ajuste(self.vistas[i].natural).min(1.0),
            Encaje::AlHueco => ajuste(self.vistas[i].natural),
            Encaje::Comun => self
                .vistas
                .iter()
                .filter(|v| v.visible)
                .map(|v| ajuste(v.natural))
                .fold(f32::INFINITY, f32::min),
        };
        (base * self.zoom_de(i)).clamp(0.05, 8.0)
    }

    fn textura(&mut self, ctx: &egui::Context, i: usize, escala: f32) -> egui::TextureHandle {
        // la caché es por pasos de escala: repintar en cada píxel de arrastre
        // rasterizaría el SVG entero decenas de veces por segundo
        let paso = (escala * 8.0).round().max(1.0) as u32;
        let clave = (self.vistas[i].id.clone(), self.vistas[i].generacion, paso);
        if let Some(t) = self.texturas.get(&clave) {
            return t.clone();
        }
        let imagen = rasteriza(&self.vistas[i].arbol, paso as f32 / 8.0);
        let t = ctx.load_texture(
            format!("{}#{}", clave.0, paso),
            imagen,
            egui::TextureOptions::LINEAR,
        );
        self.texturas.insert(clave, t.clone());
        t
    }

    /// Simula un `show` sobre un id que ya existe, que es la pregunta del
    /// ticket: qué pasa mientras el usuario está mirando esa Vista.
    fn reemplaza(&mut self, i: usize) {
        let Some(recambio) = &self.recambio else { return };
        let nuevo = recambio.clone();
        self.contador_show += 1;
        let n = self.contador_show;
        let v = &mut self.vistas[i];
        v.natural = tamano(&nuevo);
        v.arbol = nuevo;
        v.generacion += 1;
        v.ultimo_show = n;
        v.marca = 2.5;
        if self.al_reemplazar == AlReemplazar::Salta {
            self.ir_a = Some(i);
            self.activa = i;
        }
    }
}

// ------------------------------------------------------------------- pintado

/// Pinta una vista dentro del hueco dado. Devuelve el rect ocupado.
fn pinta_vista(
    app: &mut Maqueta,
    ui: &mut egui::Ui,
    i: usize,
    hueco: egui::Vec2,
    con_nombre: bool,
) -> egui::Response {
    let escala = app.escala(i, hueco);
    let tex = app.textura(ui.ctx(), i, escala);
    let tam = app.vistas[i].natural * escala;
    let marcada = app.vistas[i].marca > 0.0;

    let marco = egui::Frame::group(ui.style())
        .fill(egui::Color32::WHITE)
        .stroke(if marcada {
            egui::Stroke::new(3.0, egui::Color32::from_rgb(0xE8, 0x7A, 0x00))
        } else {
            egui::Stroke::new(1.0, egui::Color32::from_gray(180))
        });

    marco
        .show(ui, |ui| {
            ui.vertical(|ui| {
                if con_nombre {
                    ui.horizontal(|ui| {
                        // el id ES el nombre visible: no hay segundo título
                        ui.strong(&app.vistas[i].id);
                        ui.weak(format!(
                            "{}×{} · {:.0}%",
                            app.vistas[i].natural.x as i32,
                            app.vistas[i].natural.y as i32,
                            escala * 100.0
                        ));
                        if marcada {
                            ui.colored_label(egui::Color32::from_rgb(0xE8, 0x7A, 0x00), "recién actualizada");
                        }
                    });
                }
                ui.add(egui::Image::new(&tex).fit_to_exact_size(tam));
            })
        })
        .response
}

/// La barra de pestañas tal como se vería en el producto. Lo único que cambia
/// entre los tres escenarios de cierre es lo que aquí se dibuja: la ✕ existe o
/// no, y el rastro de las que ya se fueron existe o no.
fn barra_de_pestanas(app: &mut Maqueta, ui: &mut egui::Ui, idx: &[usize]) {
    let acento = egui::Color32::from_rgb(0x3B, 0x82, 0xF6);
    ui.horizontal_wrapped(|ui| {
        // en el escenario del tope, las que cayeron dejan rastro y nada más
        if app.cierre == Cierre::Tope && app.retiradas > 0 {
            ui.weak(format!("{} retiradas", app.retiradas));
            ui.add_space(6.0);
        }
        let mut cerrar = None;
        for &i in idx {
            let activa = app.activa == i;
            let relleno = if activa {
                acento.gamma_multiply(0.22)
            } else {
                egui::Color32::TRANSPARENT
            };
            let marco = egui::Frame::new()
                .fill(relleno)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .corner_radius(6.0);
            marco.show(ui, |ui| {
                ui.horizontal(|ui| {
                    // el id ES el nombre visible: no hay segundo título
                    let texto = egui::RichText::new(&app.vistas[i].id);
                    let texto = if activa { texto.strong() } else { texto };
                    if ui.add(egui::Label::new(texto).sense(egui::Sense::click())).clicked() {
                        app.activa = i;
                    }
                    if app.vistas[i].marca > 0.0 {
                        ui.colored_label(egui::Color32::from_rgb(0xE8, 0x7A, 0x00), "•");
                    }
                    // sólo un escenario le da mando al usuario sobre la Pizarra
                    if app.cierre == Cierre::Usuario && ui.small_button("×").clicked() {
                        cerrar = Some(i);
                    }
                });
            });
        }
        if let Some(i) = cerrar {
            app.vistas[i].visible = false;
            app.retiradas += 1;
            if app.activa == i {
                app.activa = *idx.iter().find(|&&j| j != i).unwrap_or(&i);
            }
        }
    });
}

fn barra(app: &mut Maqueta, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.strong("disposición:");
        for d in Disposicion::TODAS {
            ui.selectable_value(&mut app.disposicion, d, d.nombre());
        }
        ui.separator();
        ui.strong("encaje:");
        for e in Encaje::TODOS {
            ui.selectable_value(&mut app.encaje, e, e.nombre());
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.strong("zoom:");
        ui.selectable_value(&mut app.zoom_compartido, true, "de la ventana");
        ui.selectable_value(&mut app.zoom_compartido, false, "por vista");
        if app.zoom_compartido {
            ui.add(egui::Slider::new(&mut app.zoom_ventana, 0.2..=4.0).text("×"));
        } else {
            ui.weak("(rueda sobre cada vista)");
        }
        ui.separator();
        ui.strong("orden:");
        for o in Orden::TODOS {
            ui.selectable_value(&mut app.orden, o, o.nombre());
        }
        ui.separator();
        ui.checkbox(&mut app.mostrar_nombres, "nombres");
    });
    ui.horizontal_wrapped(|ui| {
        ui.strong("cómo muere una Vista:");
        for c in Cierre::TODOS {
            ui.selectable_value(&mut app.cierre, c, c.nombre());
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.strong("vistas vivas:");
        for i in 0..app.vistas.len() {
            let mut v = app.vistas[i].visible;
            let id = app.vistas[i].id.clone();
            if ui.checkbox(&mut v, id).changed() {
                app.vistas[i].visible = v;
            }
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.strong("al reemplazar:");
        for r in AlReemplazar::TODOS {
            ui.selectable_value(&mut app.al_reemplazar, r, r.nombre());
        }
        ui.separator();
        if ui.button("show sobre «actual»").clicked() {
            if let Some(i) = app.vistas.iter().position(|v| v.id == "actual") {
                app.reemplaza(i);
            }
        }
    });
}

fn cuerpo(app: &mut Maqueta, ui: &mut egui::Ui) {
    let idx = app.indices();
    let disponible = ui.available_size();

    match app.disposicion {
        Disposicion::Pestanas => {
            barra_de_pestanas(app, ui, &idx);
            ui.separator();
            let hueco = ui.available_size();
            egui::ScrollArea::both().show(ui, |ui| {
                ui.set_min_width(hueco.x);
                let a = if idx.contains(&app.activa) {
                    app.activa
                } else {
                    *idx.first().unwrap_or(&0)
                };
                // una sola Vista a la vista: centrada, que es donde miran los ojos
                ui.vertical_centered(|ui| {
                    pinta_vista(app, ui, a, hueco, false);
                });
            });
        }

        Disposicion::Columnas => {
            let n = idx.len().max(1);
            let gap = 12.0;
            let celda = egui::vec2(
                (disponible.x - gap * n as f32) / n as f32,
                disponible.y - gap,
            );
            egui::ScrollArea::both().show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    for &i in &idx {
                        let dentro = celda
                            - egui::vec2(24.0, if app.mostrar_nombres { 56.0 } else { 24.0 });
                        ui.allocate_ui(celda, |ui| {
                            pinta_vista(app, ui, i, dentro, app.mostrar_nombres);
                        });
                    }
                });
            });
        }

        Disposicion::Rejilla => {
            let n = idx.len().max(1);
            let cols = (n as f32).sqrt().ceil() as usize;
            let filas = (n + cols - 1) / cols;
            let gap = 12.0;
            let celda = egui::vec2(
                (disponible.x - gap * cols as f32) / cols as f32,
                (disponible.y - gap * filas as f32) / filas as f32,
            );
            egui::ScrollArea::vertical().show(ui, |ui| {
                for fila in idx.chunks(cols) {
                    ui.horizontal_top(|ui| {
                        for &i in fila {
                            let dentro = celda - egui::vec2(24.0, if app.mostrar_nombres { 56.0 } else { 24.0 });
                            ui.allocate_ui(celda, |ui| {
                                pinta_vista(app, ui, i, dentro, app.mostrar_nombres);
                            });
                        }
                    });
                }
            });
        }

        Disposicion::Apiladas | Disposicion::Mezcla => {
            let maximizada = if app.disposicion == Disposicion::Mezcla {
                app.maximizada
            } else {
                None
            };
            let ir_a = app.ir_a.take();
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(m) = maximizada {
                    if ui.button("⤡ volver a la pila").clicked() {
                        app.maximizada = None;
                    }
                    pinta_vista(app, ui, m, disponible, app.mostrar_nombres);
                    return;
                }
                for &i in &idx {
                    // en la pila la altura no la impone nadie: manda el ancho
                    let hueco = egui::vec2(ui.available_width(), f32::INFINITY);
                    let r = pinta_vista(app, ui, i, hueco, app.mostrar_nombres);
                    if app.disposicion == Disposicion::Mezcla
                        && r.interact(egui::Sense::click()).clicked()
                    {
                        app.maximizada = Some(i);
                    }
                    if !app.zoom_compartido {
                        let hover = ui.rect_contains_pointer(r.rect);
                        if hover {
                            let d = ui.ctx().input(|s| s.smooth_scroll_delta.y);
                            if d != 0.0 {
                                app.vistas[i].zoom = (app.vistas[i].zoom
                                    * (1.0 + d * 0.002))
                                    .clamp(0.1, 6.0);
                            }
                        }
                    }
                    if ir_a == Some(i) {
                        r.scroll_to_me(Some(egui::Align::Center));
                    }
                    ui.add_space(10.0);
                }
            });
        }
    }
}

impl eframe::App for Maqueta {
    fn ui(&mut self, ui: &mut egui::Ui, _f: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let dt = ctx.input(|s| s.stable_dt).min(0.1);
        for v in &mut self.vistas {
            if v.marca > 0.0 {
                v.marca -= dt;
                ctx.request_repaint();
            }
        }

        if !self.limpio {
            egui::Panel::top(egui::Id::new("barra")).show(ui, |ui| barra(self, ui));
        }
        egui::CentralPanel::default().show(ui, |ui| cuerpo(self, ui));

        if let Some(dir) = self.captura.clone() {
            if self.guion {
                self.rueda_escenarios(&ctx, &dir);
            } else {
                self.rueda_captura(&ctx, &dir);
            }
        }
        self.frames += 1;
    }
}

impl Maqueta {
    /// Recorre las disposiciones pintando cada una y guardando un PNG, dos
    /// veces: con las cinco Vistas y con sólo el par del caso protagonista.
    /// Sin esto no habría forma de mirar las maquetas sin estar delante.
    fn rueda_captura(&mut self, ctx: &egui::Context, dir: &Path) {
        const ESPERA: u64 = 6; // frames por maqueta antes de disparar
        const MAQUETAS: usize = 2 * Disposicion::TODAS.len();

        let paso = (self.frames / ESPERA) as usize;
        if paso > MAQUETAS {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if paso < MAQUETAS {
            let (ronda, i) = (paso / Disposicion::TODAS.len(), paso % Disposicion::TODAS.len());
            self.disposicion = Disposicion::TODAS[i];
            self.maximizada = (self.disposicion == Disposicion::Mezcla).then_some(0);
            // la segunda ronda deja vivas sólo "actual" y "propuesto"
            for v in &mut self.vistas {
                let par = self.pares.contains(&v.id);
                v.visible = ronda == 0 || par;
            }
        }
        ctx.request_repaint();

        if self.frames % ESPERA == ESPERA - 1 && paso < MAQUETAS {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let disparo: Vec<std::sync::Arc<egui::ColorImage>> = ctx.input(|s| {
            s.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
                .collect()
        });
        for img in disparo {
            let hecho = paso.saturating_sub(1);
            let ronda = if hecho < Disposicion::TODAS.len() { "5-vistas" } else { "2-vistas" };
            let nombre = Disposicion::TODAS[hecho % Disposicion::TODAS.len()].nombre();
            let ruta = dir.join(format!("{ronda}-{nombre}.png"));
            let (w, h) = (img.size[0] as u32, img.size[1] as u32);
            if let Some(buf) = image::RgbaImage::from_raw(w, h, img.as_raw().to_vec()) {
                let _ = buf.save(&ruta);
                eprintln!("captura {}", ruta.display());
            }
        }
    }
}

/// Una toma del guion: el estado exacto de la Pizarra que hay que retratar.
struct Toma {
    nombre: &'static str,
    cierre: Cierre,
    vivas: &'static [&'static str],
    activa: &'static str,
    retiradas: usize,
}

/// Los tres caminos por los que una Vista puede morir, antes y después. Lo que
/// se retrata no es la mecánica: es lo que el usuario ve de ella.
const GUION: [Toma; 7] = [
    Toma {
        nombre: "1-usuario-antes",
        cierre: Cierre::Usuario,
        vivas: &["actual", "propuesto", "transporte", "arranque", "zip-vs-repo", "hilos", "variantes-visor"],
        activa: "zip-vs-repo",
        retiradas: 0,
    },
    Toma {
        nombre: "2-usuario-despues",
        cierre: Cierre::Usuario,
        vivas: &["propuesto", "zip-vs-repo", "hilos", "variantes-visor"],
        activa: "zip-vs-repo",
        retiradas: 3,
    },
    Toma {
        nombre: "3-agente-antes",
        cierre: Cierre::Agente,
        vivas: &["actual", "propuesto", "transporte", "arranque", "zip-vs-repo", "hilos", "variantes-visor"],
        activa: "zip-vs-repo",
        retiradas: 0,
    },
    Toma {
        // el agente retira la que el usuario estaba mirando
        nombre: "4-agente-despues",
        cierre: Cierre::Agente,
        vivas: &["actual", "propuesto", "hilos", "variantes-visor"],
        activa: "hilos",
        retiradas: 0,
    },
    Toma {
        nombre: "5-tope-antes",
        cierre: Cierre::Tope,
        vivas: &["arranque", "zip-vs-repo", "hilos", "variantes-visor"],
        activa: "arranque",
        retiradas: 3,
    },
    Toma {
        // la decisión firmada: el agente conduce, así que su `show` queda delante
        nombre: "7-show-trae-al-frente",
        cierre: Cierre::Agente,
        vivas: &["actual", "propuesto", "hilos", "variantes-visor", "decision-nueva"],
        activa: "decision-nueva",
        retiradas: 0,
    },
    Toma {
        // llega una Vista nueva y cae la más vieja, que era la abierta
        nombre: "6-tope-despues",
        cierre: Cierre::Tope,
        vivas: &["zip-vs-repo", "hilos", "variantes-visor", "decision-nueva"],
        activa: "zip-vs-repo",
        retiradas: 4,
    },
];

impl Maqueta {
    /// Recorre el guion retratando cada toma. Es la respuesta a "cómo quedaría
    /// en la realidad": la misma ventana, sin interruptores por encima.
    fn rueda_escenarios(&mut self, ctx: &egui::Context, dir: &Path) {
        const ESPERA: u64 = 6;
        let paso = (self.frames / ESPERA) as usize;
        if paso > GUION.len() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if paso < GUION.len() {
            let t = &GUION[paso];
            self.disposicion = Disposicion::Pestanas;
            self.cierre = t.cierre;
            self.retiradas = t.retiradas;
            for i in 0..self.vistas.len() {
                self.vistas[i].visible = t.vivas.contains(&self.vistas[i].id.as_str());
                if self.vistas[i].id == t.activa {
                    self.activa = i;
                }
            }
        }
        ctx.request_repaint();

        if self.frames % ESPERA == ESPERA - 1 && paso < GUION.len() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let disparo: Vec<std::sync::Arc<egui::ColorImage>> = ctx.input(|s| {
            s.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
                .collect()
        });
        for img in disparo {
            let ruta = dir.join(format!("{}.png", GUION[paso.saturating_sub(1)].nombre));
            let (w, h) = (img.size[0] as u32, img.size[1] as u32);
            if let Some(buf) = image::RgbaImage::from_raw(w, h, img.as_raw().to_vec()) {
                let _ = buf.save(&ruta);
                eprintln!("captura {}", ruta.display());
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let captura = args
        .iter()
        .position(|a| a == "--captura")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);
    if let Some(d) = &captura {
        let _ = std::fs::create_dir_all(d);
    }

    // --vistas deja mirar otra Pizarra: p.ej. N variantes del mismo enfoque
    let raiz = args
        .iter()
        .position(|a| a == "--vistas")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("vistas"));
    let mut app = Maqueta::nueva(&raiz, captura);
    // por defecto la comparación del caso protagonista; con otro set, las dos primeras
    app.pares = ["actual", "propuesto"]
        .iter()
        .map(|s| s.to_string())
        .filter(|s| app.vistas.iter().any(|v| &v.id == s))
        .collect();
    if let Some(e) = args.iter().position(|a| a == "--encaje").and_then(|i| args.get(i + 1)) {
        app.encaje = match e.as_str() {
            "natural" => Encaje::Natural,
            "hueco" => Encaje::AlHueco,
            "comun" => Encaje::Comun,
            _ => Encaje::SinAgrandar,
        };
    }
    app.limpio = args.iter().any(|a| a == "--limpio");
    app.guion = args.iter().any(|a| a == "--guion");
    if let Some(c) = args.iter().position(|a| a == "--escenario").and_then(|i| args.get(i + 1)) {
        app.cierre = match c.as_str() {
            "agente" => Cierre::Agente,
            "tope" => Cierre::Tope,
            _ => Cierre::Usuario,
        };
    }
    if app.pares.is_empty() {
        app.pares = app.vistas.iter().take(2).map(|v| v.id.clone()).collect();
    }

    let opciones = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 950.0])
            .with_title("Maqueta — N vistas en una ventana"),
        ..Default::default()
    };
    eframe::run_native("n-vistas", opciones, Box::new(|_cc| Ok(Box::new(app))))
}
