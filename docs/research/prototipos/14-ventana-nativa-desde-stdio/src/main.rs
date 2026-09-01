//! Spike: ¿puede un servidor MCP por stdio (proceso hijo, sin app bundle)
//! abrir una ventana nativa con foco, y seguir sirviendo stdio mientras tanto?
//!
//! Reparto de hilos que se pone a prueba:
//!   hilo principal  -> event loop de winit/egui (obligatorio en macOS)
//!   hilo secundario -> lector de stdin, hace de "servidor" (aquí, JSON por líneas)
//!
//! Uso: pizarra-spike [--policy default|accessory|regular] [--eager]
//!   --policy   política de activación con la que arranca la app
//!   --eager    abre la ventana al arrancar en vez de al primer `show`

use std::io::{BufRead, Write};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use eframe::egui;
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

/// Lo que el hilo principal sabe de sí mismo y el hilo del servidor puede contar.
#[derive(Default, Clone)]
struct Status {
    /// política de activación efectiva, leída de AppKit
    policy: String,
    /// ¿es esta app la que tiene el foco del sistema?
    active: bool,
    visible: bool,
    views: usize,
    frames: u64,
    /// instante (segundos desde el arranque) del último frame pintado
    last_frame: f64,
}

/// Lo que el hilo del servidor le manda al event loop.
enum Cmd {
    Show { view: String, nodes: usize },
    Clear,
    /// stdin cerrado: el host ha muerto.
    HostGone,
}

fn log(msg: &str) {
    let mut err = std::io::stderr();
    let _ = writeln!(err, "[spike {:>8.3}s] {msg}", elapsed());
    let _ = err.flush();
}

fn elapsed() -> f64 {
    use std::sync::OnceLock;
    static T0: OnceLock<Instant> = OnceLock::new();
    T0.get_or_init(Instant::now).elapsed().as_secs_f64()
}

fn main() -> eframe::Result<()> {
    elapsed();
    let args: Vec<String> = std::env::args().collect();
    let policy = match arg_value(&args, "--policy").as_deref() {
        Some("accessory") => Some(ActivationPolicy::Accessory),
        Some("regular") => Some(ActivationPolicy::Regular),
        Some("prohibited") => Some(ActivationPolicy::Prohibited),
        _ => None,
    };
    let eager = args.iter().any(|a| a == "--eager");
    let no_app_nap = args.iter().any(|a| a == "--no-app-nap");
    // Se mantiene viva la referencia: App Nap vuelve en cuanto se suelta.
    let _activity = if no_app_nap { Some(disable_app_nap()) } else { None };
    log(&format!(
        "arranca pid={} policy={:?} eager={eager}",
        std::process::id(),
        arg_value(&args, "--policy")
    ));

    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Pizarra (spike)")
            .with_inner_size([720.0, 480.0])
            .with_visible(eager),
        ..Default::default()
    };
    if let Some(policy) = policy {
        options.event_loop_builder = Some(Box::new(move |builder| {
            builder.with_activation_policy(policy);
        }));
    }

    eframe::run_native(
        "pizarra-spike",
        options,
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();
            let (tx, rx) = channel::<Cmd>();
            let status = Arc::new(Mutex::new(Status::default()));
            let status_for_server = status.clone();
            std::thread::spawn(move || server_thread(tx, ctx, status_for_server));
            log("event loop en marcha, ventana creada (oculta si no --eager)");
            Ok(Box::new(App::new(rx, eager, status)))
        }),
    )
}

/// Le pide a macOS que no duerma este proceso (App Nap). El objeto devuelto
/// hay que conservarlo: al soltarlo, la actividad termina.
fn disable_app_nap() -> objc2::rc::Retained<objc2::runtime::AnyObject> {
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    // NSActivityUserInitiated | NSActivityLatencyCritical
    const OPTIONS: u64 = 0x00FFFFFF | (1 << 20) | 0xFF00000000;
    unsafe {
        let info: *mut AnyObject = msg_send![class!(NSProcessInfo), processInfo];
        let reason: Retained<AnyObject> = {
            let s: *mut AnyObject = msg_send![class!(NSString), alloc];
            let bytes = b"pizarra viva mientras dure la sesion MCP\0";
            let s: *mut AnyObject = msg_send![s, initWithUTF8String: bytes.as_ptr()];
            Retained::from_raw(s).unwrap()
        };
        let activity: *mut AnyObject =
            msg_send![info, beginActivityWithOptions: OPTIONS, reason: &*reason];
        log("App Nap deshabilitado (beginActivityWithOptions)");
        Retained::retain(activity).unwrap()
    }
}

fn arg_value(args: &[String], name: &str) -> Option<String> {
    let i = args.iter().position(|a| a == name)?;
    args.get(i + 1).cloned()
}

/// Hace de servidor MCP: lee líneas JSON de stdin, responde por stdout.
/// Corre en un hilo secundario mientras el event loop tiene el principal.
fn server_thread(tx: Sender<Cmd>, ctx: egui::Context, status: Arc<Mutex<Status>>) {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) else {
            log(&format!("json ilegible: {line}"));
            continue;
        };
        let id = msg.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let result = match method {
            "show" => {
                let view = msg
                    .pointer("/params/view")
                    .and_then(|v| v.as_str())
                    .unwrap_or("sin nombre")
                    .to_string();
                let nodes = msg
                    .pointer("/params/nodes")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3) as usize;
                let _ = tx.send(Cmd::Show { view: view.clone(), nodes });
                ctx.request_repaint();
                serde_json::json!({ "shown": view })
            }
            "clear" => {
                let _ = tx.send(Cmd::Clear);
                ctx.request_repaint();
                serde_json::json!({ "cleared": true })
            }
            "ping" => {
                let s = status.lock().unwrap().clone();
                serde_json::json!({
                    "pong": true,
                    "policy": s.policy,
                    "active": s.active,
                    "visible": s.visible,
                    "views": s.views,
                    "frames": s.frames,
                    "last_frame": s.last_frame,
                    "now": elapsed(),
                    "stale_ms": ((elapsed() - s.last_frame) * 1000.0).round(),
                })
            }
            other => serde_json::json!({ "error": format!("método desconocido: {other}") }),
        };
        let reply = serde_json::json!({ "id": id, "result": result });
        let _ = writeln!(out, "{reply}");
        let _ = out.flush();
    }
    log("stdin cerrado: el host ha muerto");
    let _ = tx.send(Cmd::HostGone);
    ctx.request_repaint();
    if std::env::args().any(|a| a == "--hard-exit") {
        // Cinturón: el reloj de la muerte corre en ESTE hilo, no en el event
        // loop -- que macOS congela cuando la ventana está tapada.
        std::thread::sleep(std::time::Duration::from_secs(3));
        log("hard-exit: salgo sin esperar al event loop");
        std::process::exit(0);
    }
}

struct App {
    rx: Receiver<Cmd>,
    status: Arc<Mutex<Status>>,
    views: Vec<(String, usize)>,
    /// true en cuanto ha habido al menos un `show` (o `clear`) en esta sesión
    used: bool,
    visible: bool,
    host_gone: bool,
    host_gone_at: Option<Instant>,
    frames: u64,
}

impl App {
    fn new(rx: Receiver<Cmd>, eager: bool, status: Arc<Mutex<Status>>) -> Self {
        Self {
            rx,
            status,
            views: Vec::new(),
            used: false,
            visible: eager,
            host_gone: false,
            host_gone_at: None,
            frames: 0,
        }
    }

    /// Lee de AppKit lo que el proceso es *ahora mismo*, no lo que creemos.
    fn refresh_status(&mut self) {
        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
        let Some(mtm) = objc2::MainThreadMarker::new() else { return };
        let app = NSApplication::sharedApplication(mtm);
        let policy = match app.activationPolicy() {
            NSApplicationActivationPolicy::Regular => "Regular",
            NSApplicationActivationPolicy::Accessory => "Accessory",
            NSApplicationActivationPolicy::Prohibited => "Prohibited",
            _ => "?",
        };
        let mut s = self.status.lock().unwrap();
        s.policy = policy.to_string();
        s.active = app.isActive();
        s.visible = self.visible;
        s.views = self.views.len();
        s.frames = self.frames;
        s.last_frame = elapsed();
    }

    /// Sube la política de activación a Regular y trae la app al frente.
    /// Es lo que convierte un proceso accesorio en una app con Dock y foco.
    fn become_foreground(&self) {
        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
        let mtm = objc2::MainThreadMarker::new()
            .expect("become_foreground debe correr en el hilo principal");
        let app = NSApplication::sharedApplication(mtm);
        let changed = app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
        app.activate();
        log(&format!("setActivationPolicy(Regular) -> {changed}, activate()"));
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let ctx = &ctx;
        self.frames += 1;
        self.refresh_status();
        while let Ok(cmd) = self.rx.try_recv() {
            match cmd {
                Cmd::Show { view, nodes } => {
                    self.used = true;
                    if let Some(v) = self.views.iter_mut().find(|(n, _)| *n == view) {
                        v.1 = nodes;
                    } else {
                        self.views.push((view, nodes));
                    }
                    if !self.visible {
                        self.visible = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                        self.become_foreground();
                        log("primer show: ventana visible + foco pedido");
                    }
                }
                Cmd::Clear => {
                    self.used = true;
                    self.views.clear();
                }
                Cmd::HostGone => {
                    self.host_gone = true;
                    self.host_gone_at = Some(Instant::now());
                    log("host muerto: estado terminal");
                }
            }
        }

        {
            if self.host_gone {
                ui.heading("Sesión terminada");
                ui.label("La conversación que motivó esta pizarra ha acabado.");
                let left = 3.0 - self.host_gone_at.map(|t| t.elapsed().as_secs_f32()).unwrap_or(0.0);
                ui.label(format!("Se cierra sola en {:.1}s", left.max(0.0)));
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
                if left <= 0.0 {
                    log("cierro la ventana y salgo");
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                return;
            }
            if self.views.is_empty() {
                ui.heading("La pizarra está vacía");
                ui.label("El agente aún no ha mostrado nada.");
                return;
            }
            ui.heading(format!("{} vista(s)", self.views.len()));
            for (name, nodes) in &self.views {
                ui.group(|ui| {
                    ui.label(egui::RichText::new(name).strong());
                    ui.horizontal_wrapped(|ui| {
                        for i in 0..*nodes {
                            let _ = ui.button(format!("nodo {i}"));
                        }
                    });
                });
            }
        }
    }
}
