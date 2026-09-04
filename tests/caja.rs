use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

use serde_json::Value;

const EMPAQUETA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/publicacion/empaqueta.sh");
const CATALOGO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/publicacion/catalogo.sh");

/// La versión del crate es el único número escrito a mano de todo el release:
/// el tag lo repite y los dos generadores se niegan si no coincide.
const LA_VERSION: &str = env!("CARGO_PKG_VERSION");

const REPO: &str = "una-cuenta/un-repo";

/// Un destino de publicación con un binario de juguete al lado. Al empaquetador
/// le da igual qué es el binario, así que tres bytes miden lo mismo que los 47
/// MB del universal y no cuestan el `cp`.
struct Banco {
    ruta: PathBuf,
}

impl Banco {
    fn nuevo() -> Self {
        static SIGUIENTE: AtomicU32 = AtomicU32::new(0);
        let ruta = std::env::temp_dir().join(format!(
            "flipchart-caja-{}-{}",
            std::process::id(),
            SIGUIENTE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&ruta).expect("el destino de la publicación se deja crear");
        fs::write(ruta.join("binario-de-juguete"), "no es un Mach-O")
            .expect("el binario de juguete se escribe");
        Self { ruta }
    }

    fn empaqueta(&self, tag: &str) -> Result<PathBuf, String> {
        let corrido = Command::new(EMPAQUETA)
            .arg(tag)
            .arg(self.ruta.join("binario-de-juguete"))
            .arg(&self.ruta)
            .output()
            .expect("el empaquetador corre");
        if corrido.status.success() {
            Ok(PathBuf::from(
                String::from_utf8_lossy(&corrido.stdout).trim(),
            ))
        } else {
            Err(String::from_utf8_lossy(&corrido.stderr).trim().to_string())
        }
    }

    fn el_zip(&self) -> PathBuf {
        self.empaqueta(&format!("v{LA_VERSION}"))
            .expect("la caja se empaqueta")
    }

    fn catalogo_de(&self, tag: &str, zip: &Path) -> Result<Value, String> {
        let corrido = Command::new(CATALOGO)
            .args([tag, &zip.to_string_lossy(), REPO])
            .output()
            .expect("el generador del catálogo corre");
        if corrido.status.success() {
            Ok(serde_json::from_slice(&corrido.stdout).expect("el catálogo es JSON"))
        } else {
            Err(String::from_utf8_lossy(&corrido.stderr).trim().to_string())
        }
    }

    fn catalogo(&self) -> Value {
        let zip = self.el_zip();
        self.catalogo_de(&format!("v{LA_VERSION}"), &zip)
            .expect("el catálogo se genera")
    }
}

impl Drop for Banco {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.ruta);
    }
}

/// Lo que `zipinfo` dice de cada entrada del zip: modo y nombre, que son las dos
/// cosas de las que depende que el binario llegue ejecutable.
fn dentro_del_zip(zip: &Path) -> Vec<(String, String)> {
    let mirado = Command::new("unzip")
        .arg("-Z")
        .arg(zip)
        .output()
        .expect("zipinfo corre");
    let listado = String::from_utf8_lossy(&mirado.stdout);
    let mut entradas: Vec<(String, String)> = listado
        .lines()
        .filter(|linea| linea.starts_with('-') || linea.starts_with('d'))
        .map(|linea| {
            let campos: Vec<&str> = linea.split_whitespace().collect();
            (campos[0].to_string(), campos[campos.len() - 1].to_string())
        })
        .collect();
    entradas.sort_by(|uno, otro| uno.1.cmp(&otro.1));
    entradas
}

fn del_zip(zip: &Path, entrada: &str) -> String {
    let sacado = Command::new("unzip")
        .arg("-p")
        .arg(zip)
        .arg(entrada)
        .output()
        .expect("unzip corre");
    String::from_utf8_lossy(&sacado.stdout).to_string()
}

fn json_del_zip(zip: &Path, entrada: &str) -> Value {
    serde_json::from_str(&del_zip(zip, entrada)).expect("la entrada del zip es JSON")
}

fn el_manifiesto_versionado() -> Value {
    let ruta =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("publicacion/caja/.claude-plugin/plugin.json");
    serde_json::from_str(&fs::read_to_string(ruta).expect("el manifiesto de la caja se lee"))
        .expect("el manifiesto de la caja es JSON")
}

#[test]
fn la_caja_lleva_los_cuatro_ficheros_y_nada_mas() {
    let banco = Banco::nuevo();

    let nombres: Vec<String> = dentro_del_zip(&banco.el_zip())
        .into_iter()
        .filter(|(modo, _)| !modo.starts_with('d'))
        .map(|(_, nombre)| nombre)
        .collect();

    assert_eq!(
        nombres,
        [
            ".claude-plugin/plugin.json",
            ".mcp.json",
            "flipchart",
            "launcher.sh"
        ]
    );
}

/// Lo que el host hace es `chmod(mode & 0o777)` cuando el zip trae bit de
/// ejecución, así que este modo es el que decide si el binario llega usable.
#[test]
fn el_binario_viaja_con_bit_de_ejecucion() {
    let banco = Banco::nuevo();

    let modos = dentro_del_zip(&banco.el_zip());

    assert!(modos.contains(&("-rwxr-xr-x".to_string(), "flipchart".to_string())));
}

#[test]
fn el_lanzador_viaja_con_bit_de_ejecucion() {
    let banco = Banco::nuevo();

    let modos = dentro_del_zip(&banco.el_zip());

    assert!(modos.contains(&("-rwxr-xr-x".to_string(), "launcher.sh".to_string())));
}

/// Un binario sin permiso de ejecución sí lo arregla el Lanzador; un binario
/// puesto de `command` en vez del Lanzador se lleva por delante el Servidor de
/// aviso, que es la única voz que queda cuando el binario no sirve.
#[test]
fn el_command_del_mcp_json_es_el_lanzador() {
    let banco = Banco::nuevo();

    let mcp = json_del_zip(&banco.el_zip(), ".mcp.json");

    assert_eq!(
        mcp["mcpServers"]["flipchart"]["command"],
        "${CLAUDE_PLUGIN_ROOT}/launcher.sh"
    );
}

/// La UI de `/plugin` hace `manifest.version ?? "unknown"`, y en la única
/// pantalla donde el usuario juzga si se fía de un binario nativo sin notarizar
/// pondría `unknown`.
#[test]
fn el_manifiesto_de_la_caja_declara_la_version() {
    let banco = Banco::nuevo();

    let manifiesto = json_del_zip(&banco.el_zip(), ".claude-plugin/plugin.json");

    assert_eq!(manifiesto["version"], LA_VERSION);
}

#[test]
fn el_manifiesto_versionado_declara_la_version_del_crate() {
    assert_eq!(el_manifiesto_versionado()["version"], LA_VERSION);
}

#[test]
fn el_zip_se_llama_por_la_version_que_lleva_dentro() {
    let banco = Banco::nuevo();

    let zip = banco.el_zip();

    assert_eq!(
        zip.file_name().unwrap().to_string_lossy(),
        format!("flipchart-{LA_VERSION}.zip")
    );
}

#[test]
fn un_tag_que_no_casa_con_lo_declarado_no_se_empaqueta() {
    let banco = Banco::nuevo();

    let fallo = banco
        .empaqueta("v9.9.9")
        .expect_err("el empaquetador se niega");

    assert!(fallo.ends_with("y el tag dice 9.9.9"), "{fallo}");
}

#[test]
fn sin_binario_no_se_empaqueta() {
    let banco = Banco::nuevo();
    fs::remove_file(banco.ruta.join("binario-de-juguete")).expect("el binario se deja borrar");

    let fallo = banco
        .empaqueta(&format!("v{LA_VERSION}"))
        .expect_err("el empaquetador se niega");

    assert!(fallo.starts_with("empaqueta: no hay binario en"), "{fallo}");
}

/// Medido: el `sha256` es opcional en el esquema del host y una entrada sin él
/// se instala igual y sin comprobar nada, sin aviso. Es la única defensa de
/// integridad del vehículo y se pierde en silencio.
#[test]
fn el_catalogo_declara_el_sha256_del_zip_que_publica() {
    let banco = Banco::nuevo();
    let zip = banco.el_zip();
    let suyo = Command::new("shasum")
        .args(["-a", "256"])
        .arg(&zip)
        .output()
        .expect("shasum corre");
    let esperado = String::from_utf8_lossy(&suyo.stdout)
        .split_whitespace()
        .next()
        .expect("shasum dice el digesto")
        .to_string();

    let catalogo = banco
        .catalogo_de(&format!("v{LA_VERSION}"), &zip)
        .expect("el catálogo se genera");

    assert_eq!(catalogo["plugins"][0]["source"]["sha256"], esperado);
}

/// Un digest pinneado apunta a un byte exacto, así que la URL tiene que ser la
/// del asset del release —inmutable— y no una que pueda cambiar de contenido.
#[test]
fn el_catalogo_apunta_al_asset_del_release_del_tag() {
    let banco = Banco::nuevo();

    let catalogo = banco.catalogo();

    assert_eq!(
        catalogo["plugins"][0]["source"]["url"],
        format!(
            "https://github.com/{REPO}/releases/download/v{LA_VERSION}/flipchart-{LA_VERSION}.zip"
        )
    );
}

#[test]
fn el_catalogo_instala_por_zip_verificado_y_no_por_clon() {
    let banco = Banco::nuevo();

    let catalogo = banco.catalogo();

    assert_eq!(catalogo["plugins"][0]["source"]["source"], "archive");
}

#[test]
fn la_version_del_catalogo_sale_del_tag() {
    let banco = Banco::nuevo();

    let catalogo = banco.catalogo();

    assert_eq!(catalogo["plugins"][0]["version"], LA_VERSION);
}

/// El nombre del `install` es el del manifiesto, no el del repo, así que el que
/// el catálogo anuncia y el que el usuario teclea tienen que ser el mismo.
#[test]
fn el_catalogo_anuncia_el_plugin_con_el_nombre_de_su_manifiesto() {
    let banco = Banco::nuevo();

    let catalogo = banco.catalogo();

    assert_eq!(
        catalogo["plugins"][0]["name"],
        el_manifiesto_versionado()["name"]
    );
}

/// `/plugin update` descarga el zip entero antes de comparar identidades: un
/// catálogo que apunte a un zip con otra versión dentro se baja, se tira y no
/// avisa.
#[test]
fn un_zip_que_declara_otra_version_no_entra_en_el_catalogo() {
    let banco = Banco::nuevo();
    let zip = banco.el_zip();

    let fallo = banco
        .catalogo_de("v9.9.9", &zip)
        .expect_err("el generador se niega");

    assert_eq!(
        fallo,
        format!("catalogo: el zip declara {LA_VERSION} y el tag dice 9.9.9")
    );
}

#[test]
fn sin_zip_no_hay_catalogo() {
    let banco = Banco::nuevo();

    let fallo = banco
        .catalogo_de(&format!("v{LA_VERSION}"), Path::new("/no-existe.zip"))
        .expect_err("el generador se niega");

    assert_eq!(fallo, "catalogo: no hay zip en /no-existe.zip");
}

/// El tope del archive son 256 MiB y **no tiene válvula**: pasarse no degrada
/// nada, deja el plugin sin forma de instalarse. Lo que come el margen son las
/// dependencias, que es justo lo que nadie mira al añadir una.
#[test]
fn la_caja_cabe_en_el_tope_del_archive() {
    let banco = Banco::nuevo();

    let bytes = fs::metadata(banco.el_zip())
        .expect("el zip se deja medir")
        .len();

    assert!(bytes <= 256 * 1024 * 1024, "{bytes} bytes de archive");
}

#[test]
fn el_lanzador_del_repo_es_el_que_se_empaqueta() {
    let banco = Banco::nuevo();
    let versionado = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/launcher.sh"))
        .expect("el Lanzador del repo se lee");

    let empaquetado = del_zip(&banco.el_zip(), "launcher.sh");

    assert_eq!(empaquetado, versionado);
}

#[test]
fn el_binario_llega_entero_al_zip() {
    let banco = Banco::nuevo();

    let empaquetado = del_zip(&banco.el_zip(), "flipchart");

    assert_eq!(empaquetado, "no es un Mach-O");
}

#[test]
fn la_caja_del_repo_no_lleva_permisos_de_ejecucion_que_no_le_tocan() {
    let modo = fs::metadata(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/publicacion/caja/.mcp.json"
    ))
    .expect("el .mcp.json versionado se deja medir")
    .permissions()
    .mode();

    assert_eq!(modo & 0o111, 0);
}
