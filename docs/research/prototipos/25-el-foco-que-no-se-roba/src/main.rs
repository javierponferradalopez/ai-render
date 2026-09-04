//! Spike: ¿se puede sacar la ventana delante **sin robarle el teclado a nadie**?
//!
//! Reproduce la costura del producto —arranque diferido, `Accessory` que sube a
//! `Regular` al construir el event loop, ⌘W que oculta y no mata— y deja
//! conmutar sólo lo que se discute: **cómo aparece la ventana**.
//!
//! Uso: foco-spike --appear activate|key|regardless [--quiet-launch]
//!   activate     Visible(true) + Focus + NSApp.activate()   (lo que hace hoy el producto)
//!   key          Visible(true) + Focus, sin activate()      (para aislar quién roba qué)
//!   regardless   orderFrontRegardless, y nada más           (la apuesta del ticket)
//!
//! `--after-frame` espera a que se haya pintado un frame antes de aparecer: la
//! misma idea que `--delay-ms` pero sin número mágico — la condición es que la
//! ventana exista de verdad, no que haya pasado un rato.
//!
//! `--delay-ms N` retrasa la aparición N ms desde que el `show` llega, para
//! separar "la ventana no puede ponerse delante" de "se ha puesto delante
//! antes de que el sistema la tuviera montada".
//!
//! `--quiet-launch` desarma lo que winit hace por su cuenta al arrancar el
//! event loop: `activateIgnoringOtherApps(true)`, que roba el teclado antes de
//! que ninguna variante llegue a opinar.
//!
//! Habla JSON por líneas —`show`, `close`, `probe`—, no MCP: lo que se mide es
//! el foco, no el protocolo. `close` llama a `performClose:`, que es
//! exactamente lo que hace ⌘W.

use foco_spike::ventanas;

use std::io::{BufRead, Write};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use eframe::egui;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSWindow};
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Aparicion {
    Activate,
    Key,
    Regardless,
}

/// Lo que el hilo principal sabe de sí mismo, leído de AppKit y no supuesto.
#[derive(Default, Clone)]
struct Estado {
    policy: String,
    /// ¿es esta app la que tiene el teclado del sistema?
    app_activa: bool,
    ventana_key: bool,
    ventana_visible: bool,
    /// pasadas de `logic()`, que corren aunque no se pinte, y de `ui()`, que no.
    logicas: u64,
    frames: u64,
    sello: f64,
}

enum Orden {
    Show(String),
    /// Lo mismo que ⌘W, por la misma puerta: `performClose:`.
    Close,
}

fn main() -> eframe::Result<()> {
    elapsed();
    let aparicion = match argumento("--appear").as_deref() {
        Some("key") => Aparicion::Key,
        Some("regardless") => Aparicion::Regardless,
        _ => Aparicion::Activate,
    };
    let arranque_callado = std::env::args().any(|a| a == "--quiet-launch");
    let tras_el_primer_frame = std::env::args().any(|a| a == "--after-frame");
    let retardo = std::time::Duration::from_millis(
        argumento("--delay-ms").and_then(|v| v.parse().ok()).unwrap_or(0),
    );
    let principal = MainThreadMarker::new().expect("main() corre en el hilo principal");
    NSApplication::sharedApplication(principal)
        .setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    log(&format!(
        "arranca pid={} appear={aparicion:?} quiet_launch={arranque_callado} retardo={retardo:?} policy=Accessory",
        std::process::id()
    ));

    let (ordenes, pendientes) = channel();
    let estado = Arc::new(Mutex::new(Estado::default()));
    let despertador: Arc<OnceLock<egui::Context>> = Arc::default();
    {
        let estado = estado.clone();
        let despertador = despertador.clone();
        std::thread::spawn(move || servidor(ordenes, estado, despertador));
    }

    // Arranque diferido, como el producto: el event loop —y la subida a
    // `Regular` que lleva dentro— no existe hasta el primer `show`.
    let Some(primera) = espera_el_primer_show(&pendientes) else {
        log("la sesión murió sin pedir un solo dibujo: no hay ventana");
        return Ok(());
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Pizarra (spike del foco)")
            .with_inner_size([720.0, 480.0])
            // Oculta al nacer: `with_visible(true)` haría
            // `makeKeyAndOrderFront` antes de que la variante pueda opinar.
            .with_visible(false),
        event_loop_builder: Some(Box::new(move |builder| {
            builder.with_activation_policy(ActivationPolicy::Regular);
            if arranque_callado {
                builder.with_activate_ignoring_other_apps(false);
            }
        })),
        ..Default::default()
    };
    eframe::run_native(
        "foco-spike",
        options,
        Box::new(move |cc| {
            let _ = despertador.set(cc.egui_ctx.clone());
            Ok(Box::new(App::new(
                aparicion,
                retardo,
                tras_el_primer_frame,
                pendientes,
                estado,
                primera,
            )))
        }),
    )
}

fn espera_el_primer_show(pendientes: &Receiver<Orden>) -> Option<String> {
    loop {
        match pendientes.recv().ok()? {
            Orden::Show(vista) => return Some(vista),
            Orden::Close => {}
        }
    }
}

/// El hilo del servidor: contesta siempre, tenga o no ventana el otro. La sonda
/// del orden Z vive aquí a propósito — es lo único que sigue siendo verdad
/// cuando el event loop está congelado.
fn servidor(ordenes: Sender<Orden>, estado: Arc<Mutex<Estado>>, despertador: Arc<OnceLock<egui::Context>>) {
    let entrada = std::io::stdin();
    let mut salida = std::io::stdout();
    for linea in entrada.lock().lines() {
        let Ok(linea) = linea else { break };
        if linea.trim().is_empty() {
            continue;
        }
        let Ok(mensaje) = serde_json::from_str::<serde_json::Value>(&linea) else {
            log(&format!("json ilegible: {linea}"));
            continue;
        };
        let id = mensaje.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let metodo = mensaje.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let resultado = match metodo {
            "show" => {
                let vista = mensaje
                    .pointer("/params/view")
                    .and_then(|v| v.as_str())
                    .unwrap_or("sin nombre")
                    .to_string();
                let _ = ordenes.send(Orden::Show(vista.clone()));
                despierta(&despertador);
                serde_json::json!({ "shown": vista })
            }
            "close" => {
                let _ = ordenes.send(Orden::Close);
                despierta(&despertador);
                serde_json::json!({ "closed": true })
            }
            "probe" => {
                // Despierta al que dibuja: sin una pasada de `logic()` lo que
                // la app cree de sí misma se queda rancio, y el rancio no
                // distingue "no me han robado el foco" de "no he mirado".
                despierta(&despertador);
                std::thread::sleep(std::time::Duration::from_millis(4));
                sonda(&estado)
            }
            otro => serde_json::json!({ "error": format!("método desconocido: {otro}") }),
        };
        let _ = writeln!(salida, "{}", serde_json::json!({ "id": id, "result": resultado }));
        let _ = salida.flush();
    }
    log("stdin cerrado: el host ha muerto, salgo");
    std::process::exit(0);
}

fn despierta(despertador: &OnceLock<egui::Context>) {
    if let Some(ctx) = despertador.get() {
        ctx.request_repaint();
    }
}

/// Una foto del momento: lo que la app cree de sí misma, y lo que el
/// WindowServer dice de todas las ventanas de la pantalla.
fn sonda(estado: &Mutex<Estado>) -> serde_json::Value {
    let estado = estado.lock().unwrap().clone();
    let mio = std::process::id() as i32;
    let ventanas = ventanas::de_delante_hacia_atras();
    serde_json::json!({
        "now": elapsed(),
        "pid": mio,
        "policy": estado.policy,
        "app_activa": estado.app_activa,
        "ventana_key": estado.ventana_key,
        "ventana_visible": estado.ventana_visible,
        "logicas": estado.logicas,
        "frames": estado.frames,
        "estado_de": estado.sello,
        // El primero de la lista es quien tiene la pantalla, y con ella el teclado.
        "delante": ventanas.first().map(|v| serde_json::json!({ "pid": v.pid, "owner": v.owner })),
        "mi_z": ventanas.iter().position(|v| v.pid == mio),
        "en_pantalla": ventanas.iter().map(|v| serde_json::json!({
            "pid": v.pid, "owner": v.owner, "bounds": [v.bounds.0, v.bounds.1, v.bounds.2, v.bounds.3],
        })).collect::<Vec<_>>(),
    })
}

struct App {
    aparicion: Aparicion,
    retardo: std::time::Duration,
    tras_el_primer_frame: bool,
    pendientes: Receiver<Orden>,
    estado: Arc<Mutex<Estado>>,
    vistas: Vec<String>,
    /// La ventana nace al primer `show` y **renace** en el siguiente tras un
    /// ⌘W, que la oculta y no la mata.
    en_pie: bool,
    pedida: bool,
    pedida_en: Option<Instant>,
    logicas: u64,
    frames: u64,
}

impl App {
    fn new(
        aparicion: Aparicion,
        retardo: std::time::Duration,
        tras_el_primer_frame: bool,
        pendientes: Receiver<Orden>,
        estado: Arc<Mutex<Estado>>,
        primera: String,
    ) -> Self {
        Self {
            aparicion,
            retardo,
            tras_el_primer_frame,
            pendientes,
            estado,
            vistas: vec![primera],
            en_pie: false,
            pedida: true,
            pedida_en: None,
            logicas: 0,
            frames: 0,
        }
    }

    fn ventana(&self) -> Option<objc2::rc::Retained<NSWindow>> {
        let principal = MainThreadMarker::new()?;
        NSApplication::sharedApplication(principal).windows().iter().next()
    }

    /// Lee de AppKit lo que el proceso es *ahora mismo*, no lo que creemos.
    fn refresca(&mut self) {
        let Some(principal) = MainThreadMarker::new() else {
            return;
        };
        let app = NSApplication::sharedApplication(principal);
        let policy = match app.activationPolicy() {
            NSApplicationActivationPolicy::Regular => "Regular",
            NSApplicationActivationPolicy::Accessory => "Accessory",
            NSApplicationActivationPolicy::Prohibited => "Prohibited",
            _ => "?",
        };
        let ventana = self.ventana();
        let mut estado = self.estado.lock().unwrap();
        estado.policy = policy.to_string();
        estado.app_activa = app.isActive();
        estado.ventana_key = ventana.as_ref().is_some_and(|v| v.isKeyWindow());
        estado.ventana_visible = ventana.as_ref().is_some_and(|v| v.isVisible());
        estado.logicas = self.logicas;
        estado.frames = self.frames;
        estado.sello = elapsed();
    }

    /// Lo único que este spike discute: **cómo aparece la ventana**.
    fn aparece(&self, ctx: &egui::Context) {
        match self.aparicion {
            Aparicion::Activate => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                if let Some(principal) = MainThreadMarker::new() {
                    NSApplication::sharedApplication(principal).activate();
                }
                log("aparece: Visible(true) + Focus + activate()");
            }
            Aparicion::Key => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                log("aparece: Visible(true) + Focus, sin activate()");
            }
            Aparicion::Regardless => {
                match self.ventana() {
                    Some(ventana) => {
                        ventana.orderFrontRegardless();
                        log("aparece: orderFrontRegardless()");
                    }
                    None => log("aparece: no hay NSWindow todavía"),
                }
            }
        }
    }
}

impl eframe::App for App {
    /// Con la ventana oculta `eframe` no corre una pasada de egui, así que todo
    /// lo que no sea pintar vive aquí: es lo único que sigue llamándose cuando
    /// la ventana no está.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.logicas += 1;
        self.refresca();

        if ctx.input(|entrada| entrada.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.en_pie = false;
            log("⌘W: la ventana se oculta, el proceso sigue");
        }

        while let Ok(orden) = self.pendientes.try_recv() {
            match orden {
                Orden::Show(vista) => {
                    if !self.vistas.contains(&vista) {
                        self.vistas.push(vista);
                    }
                    self.pedida = true;
                }
                Orden::Close => {
                    if let Some(ventana) = self.ventana() {
                        ventana.performClose(None);
                    }
                }
            }
        }

        if self.pedida && !self.en_pie {
            let desde = *self.pedida_en.get_or_insert_with(Instant::now);
            let montada = !self.tras_el_primer_frame || self.frames > 0;
            if montada && desde.elapsed() >= self.retardo {
                self.pedida = false;
                self.pedida_en = None;
                self.en_pie = true;
                self.aparece(ctx);
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(5));
            }
        } else {
            self.pedida = false;
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frames += 1;
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(50));
        ui.heading(format!("{} vista(s)", self.vistas.len()));
        for vista in &self.vistas {
            ui.label(vista);
        }
        ui.weak(format!("{:?}", self.aparicion));
    }
}

fn argumento(nombre: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    let posicion = args.iter().position(|a| a == nombre)?;
    args.get(posicion + 1).cloned()
}

fn log(mensaje: &str) {
    let mut err = std::io::stderr();
    let _ = writeln!(err, "[spike {:>8.3}s] {mensaje}", elapsed());
    let _ = err.flush();
}

fn elapsed() -> f64 {
    static T0: OnceLock<Instant> = OnceLock::new();
    T0.get_or_init(Instant::now).elapsed().as_secs_f64()
}
