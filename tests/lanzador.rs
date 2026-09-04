use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const LANZADOR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/launcher.sh");

/// Cinco segundos es un plazo de test, no del producto: el Lanzador promete
/// milisegundos, y lo que este plazo compra es que un Lanzador callado falle en
/// vez de colgar la suite.
const PLAZO: Duration = Duration::from_secs(5);

/// El directorio del plugin tal como lo deja el host: el binario al lado del
/// Lanzador, en uno de sus cuatro estados.
struct Caja {
    ruta: PathBuf,
}

impl Caja {
    fn sin_binario() -> Self {
        Self::vacia("ausente")
    }

    /// El binario de verdad, enlazado en vez de copiado: lo que se mide es que
    /// el Lanzador le cede el sitio, no cuánto tarda un `cp` de 80 MB.
    fn con_el_binario_bueno() -> Self {
        let caja = Self::vacia("bueno");
        symlink(env!("CARGO_BIN_EXE_flipchart"), caja.binario())
            .expect("el binario de la pizarra se deja enlazar");
        caja
    }

    fn con_el_binario_sin_permiso() -> Self {
        let caja = Self::vacia("sin-permiso");
        fs::copy(env!("CARGO_BIN_EXE_flipchart"), caja.binario())
            .expect("el binario de la pizarra se deja copiar");
        caja.dale_estos_permisos(0o644);
        caja
    }

    /// El `chmod` que no puede: en un sistema de ficheros de sólo lectura no se
    /// deja montar dentro de un test, y `chflags uchg` lo reproduce igual —ni el
    /// dueño le cambia los permisos—.
    fn con_un_binario_que_no_se_deja_arreglar() -> Self {
        let caja = Self::vacia("sin-arreglo");
        fs::write(caja.binario(), "").expect("el binario falso se escribe");
        caja.dale_estos_permisos(0o644);
        caja.chflags("uchg");
        caja
    }

    /// Una cabecera Mach-O de PowerPC y nada detrás: `exec` la rechaza con
    /// `ENOEXEC`, y los ceros son lo que impide que bash la tome por un script
    /// y la intente correr.
    fn con_un_binario_de_otra_arquitectura() -> Self {
        let caja = Self::vacia("otra-arquitectura");
        let mut cabecera = vec![0u8; 96];
        cabecera[..4].copy_from_slice(&0xfeed_facfu32.to_le_bytes());
        cabecera[4..8].copy_from_slice(&0x0100_0012u32.to_le_bytes());
        fs::write(caja.binario(), cabecera).expect("el binario falso se escribe");
        caja.dale_estos_permisos(0o755);
        caja
    }

    fn vacia(estado: &str) -> Self {
        static SIGUIENTE: AtomicU32 = AtomicU32::new(0);
        let ruta = std::env::temp_dir().join(format!(
            "flipchart-lanzador-{}-{estado}-{}",
            std::process::id(),
            SIGUIENTE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&ruta).expect("la caja del plugin se deja crear");
        Self { ruta }
    }

    fn binario(&self) -> PathBuf {
        self.ruta.join("flipchart")
    }

    fn chflags(&self, banderas: &str) {
        let puesto = Command::new("chflags")
            .args(["-R", banderas])
            .arg(&self.ruta)
            .status()
            .expect("chflags corre");
        assert!(puesto.success());
    }

    fn dale_estos_permisos(&self, modo: u32) {
        fs::set_permissions(self.binario(), fs::Permissions::from_mode(modo))
            .expect("los permisos del binario se dejan poner");
    }
}

impl Drop for Caja {
    fn drop(&mut self) {
        self.chflags("nouchg");
        let _ = fs::remove_dir_all(&self.ruta);
    }
}

/// El Lanzador arrancado como lo arranca el host: por stdio y con la caja del
/// plugin en `CLAUDE_PLUGIN_ROOT`.
struct Sesion {
    proceso: Child,
    entrada: Option<ChildStdin>,
    salida: Receiver<String>,
    saludo: Value,
    siguiente_id: u64,
}

impl Sesion {
    fn abierta(caja: &Caja) -> Self {
        let mut sesion = Self::cruda(&caja.ruta);
        sesion.saludo = sesion.pide(
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "prueba", "version": "0" }
            }),
        );
        sesion.notifica("notifications/initialized");
        sesion
    }

    fn cruda(raiz: &Path) -> Self {
        let mut proceso = Command::new(LANZADOR)
            .env("CLAUDE_PLUGIN_ROOT", raiz)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("el Lanzador arranca");
        let entrada = proceso.stdin.take();
        let salida = lineas_de(proceso.stdout.take().unwrap());
        Self {
            proceso,
            entrada,
            salida,
            saludo: Value::Null,
            siguiente_id: 1,
        }
    }

    fn pide(&mut self, metodo: &str, params: Value) -> Value {
        let id = self.siguiente_id;
        self.siguiente_id += 1;
        let respuesta = self.pide_con_id(json!(id), metodo, params);
        assert_eq!(respuesta["id"], json!(id));
        respuesta["result"].clone()
    }

    fn pide_con_id(&mut self, id: Value, metodo: &str, params: Value) -> Value {
        self.manda(json!({ "jsonrpc": "2.0", "id": id, "method": metodo, "params": params }));
        self.contesta()
    }

    fn notifica(&mut self, metodo: &str) {
        self.manda(json!({ "jsonrpc": "2.0", "method": metodo }));
    }

    fn manda(&mut self, mensaje: Value) {
        let entrada = self.entrada.as_mut().expect("la sesión sigue abierta");
        writeln!(entrada, "{mensaje}").expect("el Lanzador escucha");
        entrada.flush().unwrap();
    }

    fn contesta(&self) -> Value {
        let linea = self
            .salida
            .recv_timeout(PLAZO)
            .expect("el Lanzador contesta");
        serde_json::from_str(&linea).expect("JSON-RPC legible")
    }

    fn herramientas(&mut self) -> Vec<Value> {
        self.pide("tools/list", json!({}))["tools"]
            .as_array()
            .expect("tools/list trae una lista")
            .clone()
    }

    fn nombres_de_sus_herramientas(&mut self) -> Vec<String> {
        let mut nombres: Vec<String> = self
            .herramientas()
            .iter()
            .map(|h| h["name"].as_str().unwrap().to_string())
            .collect();
        nombres.sort();
        nombres
    }

    fn cierra_su_entrada(&mut self) {
        drop(self.entrada.take());
    }

    fn recibe_sigterm(&mut self) {
        let matado = Command::new("kill")
            .args(["-TERM", &self.proceso.id().to_string()])
            .status()
            .expect("kill -TERM corre");
        assert!(matado.success());
    }

    fn sale_antes_de(&mut self, plazo: Duration) -> ExitStatus {
        let limite = Instant::now() + plazo;
        while Instant::now() < limite {
            if let Some(estado) = self.proceso.try_wait().expect("el proceso se deja mirar") {
                return estado;
            }
            sleep(Duration::from_millis(20));
        }
        panic!("el Lanzador no salió");
    }
}

impl Drop for Sesion {
    fn drop(&mut self) {
        let _ = self.proceso.kill();
        let _ = self.proceso.wait();
    }
}

fn lineas_de(salida: ChildStdout) -> Receiver<String> {
    let (envia, recibe) = channel();
    spawn(move || {
        for linea in BufReader::new(salida).lines() {
            let Ok(linea) = linea else { return };
            if envia.send(linea).is_err() {
                return;
            }
        }
    });
    recibe
}

fn el_aviso_de(sesion: &mut Sesion) -> String {
    let herramientas = sesion.herramientas();
    let [aviso] = &herramientas[..] else {
        panic!("el Servidor de aviso anuncia una sola herramienta");
    };
    aviso["description"]
        .as_str()
        .expect("la herramienta lleva descripción")
        .to_string()
}

#[test]
fn con_el_binario_bueno_el_lanzador_le_cede_el_sitio() {
    let caja = Caja::con_el_binario_bueno();
    let mut sesion = Sesion::abierta(&caja);

    assert_eq!(sesion.nombres_de_sus_herramientas(), ["clear", "show"]);
}

/// `check` no abre ventana y no habla MCP, así que sirve de testigo de que los
/// argumentos cruzaron el `exec`.
#[test]
fn el_lanzador_le_pasa_al_binario_los_argumentos_con_los_que_lo_llamaron() {
    let caja = Caja::con_el_binario_bueno();

    let corrido = Command::new(LANZADOR)
        .env("CLAUDE_PLUGIN_ROOT", &caja.ruta)
        .args(["check", "/no-existe.mmd"])
        .output()
        .expect("el Lanzador corre");

    assert!(String::from_utf8_lossy(&corrido.stdout).contains("== /no-existe.mmd"));
}

#[test]
fn un_binario_sin_permiso_de_ejecucion_se_lo_lleva_puesto_y_arranca() {
    let caja = Caja::con_el_binario_sin_permiso();
    let mut sesion = Sesion::abierta(&caja);

    assert_eq!(sesion.nombres_de_sus_herramientas(), ["clear", "show"]);
}

#[test]
fn sin_binario_contesta_al_handshake_igualmente() {
    let caja = Caja::sin_binario();

    let sesion = Sesion::abierta(&caja);

    assert_eq!(sesion.saludo["serverInfo"]["name"], json!("flipchart"));
}

#[test]
fn el_handshake_del_servidor_de_aviso_habla_la_version_que_le_hablan() {
    let caja = Caja::sin_binario();

    let sesion = Sesion::abierta(&caja);

    assert_eq!(sesion.saludo["protocolVersion"], json!("2025-06-18"));
}

#[test]
fn con_un_binario_de_otra_arquitectura_contesta_al_handshake_en_milisegundos() {
    let caja = Caja::con_un_binario_de_otra_arquitectura();

    let comienzo = Instant::now();
    let _sesion = Sesion::abierta(&caja);

    assert!(comienzo.elapsed() < Duration::from_secs(2));
}

#[test]
fn el_servidor_de_aviso_anuncia_una_sola_herramienta() {
    let caja = Caja::sin_binario();
    let mut sesion = Sesion::abierta(&caja);

    assert_eq!(sesion.nombres_de_sus_herramientas(), ["unavailable"]);
}

#[test]
fn la_herramienta_del_aviso_no_pide_argumentos() {
    let caja = Caja::sin_binario();
    let mut sesion = Sesion::abierta(&caja);

    let esquema = sesion.herramientas()[0]["inputSchema"].clone();

    assert_eq!(esquema, json!({ "type": "object", "properties": {} }));
}

#[test]
fn sin_binario_el_aviso_dice_que_falta_y_que_hay_que_reinstalar() {
    let caja = Caja::sin_binario();
    let mut sesion = Sesion::abierta(&caja);

    assert_eq!(
        el_aviso_de(&mut sesion),
        "The flipchart is not available in this session and cannot draw anything: the flipchart \
         binary is not in the plugin directory. Nothing will appear on screen, so do not offer \
         the user a diagram - explain in prose instead. Reinstalling the plugin is what brings \
         it back."
    );
}

#[test]
fn con_un_binario_de_otra_arquitectura_el_aviso_dice_que_esta_maquina_no_lo_ejecuta() {
    let caja = Caja::con_un_binario_de_otra_arquitectura();
    let mut sesion = Sesion::abierta(&caja);

    assert_eq!(
        el_aviso_de(&mut sesion),
        "The flipchart is not available in this session and cannot draw anything: this machine \
         refused to execute the flipchart binary, which is a macOS build - another platform or \
         architecture cannot run it. Nothing will appear on screen, so do not offer the user a \
         diagram - explain in prose instead. Reinstalling the plugin is what brings it back."
    );
}

#[test]
fn con_un_binario_que_no_se_deja_arreglar_el_aviso_dice_que_no_hay_permiso_de_ejecucion() {
    let caja = Caja::con_un_binario_que_no_se_deja_arreglar();
    let mut sesion = Sesion::abierta(&caja);

    assert_eq!(
        el_aviso_de(&mut sesion),
        "The flipchart is not available in this session and cannot draw anything: the flipchart \
         binary could not be given execute permission. Nothing will appear on screen, so do not \
         offer the user a diagram - explain in prose instead. Reinstalling the plugin is what \
         brings it back."
    );
}

#[test]
fn llamar_a_la_herramienta_del_aviso_vuelve_marcado_como_error() {
    let caja = Caja::sin_binario();
    let mut sesion = Sesion::abierta(&caja);

    let resultado = sesion.pide(
        "tools/call",
        json!({ "name": "unavailable", "arguments": {} }),
    );

    assert_eq!(resultado["isError"], json!(true));
}

#[test]
fn llamar_a_la_herramienta_del_aviso_devuelve_el_mismo_aviso() {
    let caja = Caja::sin_binario();
    let mut sesion = Sesion::abierta(&caja);
    let anuncio = el_aviso_de(&mut sesion);

    let resultado = sesion.pide(
        "tools/call",
        json!({ "name": "unavailable", "arguments": {} }),
    );

    assert_eq!(resultado["content"][0]["text"], json!(anuncio));
}

#[test]
fn un_id_de_texto_vuelve_tal_cual() {
    let caja = Caja::sin_binario();
    let mut sesion = Sesion::abierta(&caja);

    let respuesta = sesion.pide_con_id(json!("aviso-1"), "tools/list", json!({}));

    assert_eq!(respuesta["id"], json!("aviso-1"));
}

#[test]
fn la_notificacion_initialized_no_lleva_respuesta() {
    let caja = Caja::sin_binario();
    let mut sesion = Sesion::abierta(&caja);

    sesion.notifica("notifications/initialized");

    assert_eq!(
        sesion.pide_con_id(json!(7), "tools/list", json!({}))["id"],
        json!(7)
    );
}

#[test]
fn el_servidor_de_aviso_sale_con_cero_cuando_se_le_cierra_la_entrada() {
    let caja = Caja::sin_binario();
    let mut sesion = Sesion::abierta(&caja);

    sesion.cierra_su_entrada();

    assert_eq!(sesion.sale_antes_de(PLAZO).code(), Some(0));
}

#[test]
fn el_servidor_de_aviso_sale_con_cero_cuando_lo_matan() {
    let caja = Caja::sin_binario();
    let mut sesion = Sesion::abierta(&caja);

    sesion.recibe_sigterm();

    assert_eq!(sesion.sale_antes_de(PLAZO).code(), Some(0));
}
