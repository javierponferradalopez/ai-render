use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdout, Command, ExitStatus, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use serde_json::json;

/// Una sesión sin un solo `show`: no hay ventana de la que despedirse, así que
/// el proceso sale en cuanto el hilo del servidor se entera de la muerte.
struct Sesion {
    proceso: Child,
    salida: BufReader<ChildStdout>,
}

impl Sesion {
    /// Inicializada de verdad —con la respuesta leída—, que es lo que asegura
    /// que el hilo del servidor está en pie y escuchando las dos muertes.
    fn abierta() -> Self {
        let mut proceso = Command::new(env!("CARGO_BIN_EXE_flipchart"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("el binario de la pizarra arranca");
        let salida = BufReader::new(proceso.stdout.take().unwrap());
        let mut sesion = Self { proceso, salida };
        sesion.inicializa();
        sesion
    }

    fn inicializa(&mut self) {
        let peticion = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "prueba", "version": "0" }
            }
        });
        let entrada = self.proceso.stdin.as_mut().unwrap();
        writeln!(entrada, "{peticion}").unwrap();
        entrada.flush().unwrap();
        let mut respuesta = String::new();
        self.salida
            .read_line(&mut respuesta)
            .expect("el servidor contesta al initialize");
    }

    fn cierra_su_entrada(&mut self) {
        drop(self.proceso.stdin.take());
    }

    fn recibe_sigint(&mut self) {
        let matado = Command::new("kill")
            .args(["-INT", &self.proceso.id().to_string()])
            .status()
            .expect("kill -INT corre");
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
        panic!("el proceso sobrevivió a su sesión");
    }
}

impl Drop for Sesion {
    fn drop(&mut self) {
        let _ = self.proceso.kill();
        let _ = self.proceso.wait();
    }
}

const MARGEN: Duration = Duration::from_secs(5);

#[test]
fn el_eof_en_stdin_acaba_el_proceso() {
    let mut sesion = Sesion::abierta();

    sesion.cierra_su_entrada();

    assert!(sesion.sale_antes_de(MARGEN).success());
}

#[test]
fn el_sigint_acaba_el_proceso() {
    let mut sesion = Sesion::abierta();

    sesion.recibe_sigint();

    assert!(sesion.sale_antes_de(MARGEN).success());
}
