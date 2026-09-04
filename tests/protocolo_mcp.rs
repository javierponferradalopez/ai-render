use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

struct Sesion {
    proceso: Child,
    entrada: ChildStdin,
    salida: BufReader<ChildStdout>,
    siguiente_id: u64,
}

impl Sesion {
    fn abierta() -> Self {
        let mut proceso = Command::new(env!("CARGO_BIN_EXE_flipchart"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("el binario de la pizarra arranca");
        let entrada = proceso.stdin.take().unwrap();
        let salida = BufReader::new(proceso.stdout.take().unwrap());
        let mut sesion = Self {
            proceso,
            entrada,
            salida,
            siguiente_id: 1,
        };
        sesion.pide(
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

    fn pide(&mut self, metodo: &str, params: Value) -> Value {
        let id = self.siguiente_id;
        self.siguiente_id += 1;
        let peticion = json!({ "jsonrpc": "2.0", "id": id, "method": metodo, "params": params });
        writeln!(self.entrada, "{peticion}").unwrap();
        self.entrada.flush().unwrap();
        loop {
            let mut linea = String::new();
            self.salida
                .read_line(&mut linea)
                .expect("el servidor responde");
            let mensaje: Value = serde_json::from_str(linea.trim()).expect("JSON-RPC legible");
            if mensaje.get("id") == Some(&json!(id)) {
                return mensaje["result"].clone();
            }
        }
    }

    fn notifica(&mut self, metodo: &str) {
        let aviso = json!({ "jsonrpc": "2.0", "method": metodo });
        writeln!(self.entrada, "{aviso}").unwrap();
        self.entrada.flush().unwrap();
    }

    fn herramientas(&mut self) -> Vec<Value> {
        self.pide("tools/list", json!({}))["tools"]
            .as_array()
            .expect("tools/list trae una lista")
            .clone()
    }

    fn llama(&mut self, herramienta: &str, argumentos: Value) -> Value {
        self.pide(
            "tools/call",
            json!({ "name": herramienta, "arguments": argumentos }),
        )
    }
}

impl Drop for Sesion {
    fn drop(&mut self) {
        let _ = self.proceso.kill();
        let _ = self.proceso.wait();
    }
}

fn herramienta<'a>(herramientas: &'a [Value], nombre: &str) -> &'a Value {
    herramientas
        .iter()
        .find(|h| h["name"] == json!(nombre))
        .unwrap_or_else(|| panic!("la herramienta {nombre} está registrada"))
}

fn texto(resultado: &Value) -> &str {
    resultado["content"][0]["text"].as_str().expect("un texto")
}

#[test]
fn el_servidor_expone_show_y_clear_y_nada_mas() {
    let mut sesion = Sesion::abierta();

    let mut nombres: Vec<String> = sesion
        .herramientas()
        .iter()
        .map(|h| h["name"].as_str().unwrap().to_string())
        .collect();
    nombres.sort();

    assert_eq!(nombres, ["clear", "show"]);
}

#[test]
fn show_lleva_la_descripcion_literal_del_5_3() {
    let mut sesion = Sesion::abierta();

    let show = herramienta(&sesion.herramientas(), "show").clone();

    assert_eq!(
        show["description"],
        json!(
            "Show a diagram on the ephemeral flipchart window, as a named view. Takes Mermaid source.\n\n\
             Any id used in a relationship must carry a label or a body when another id in the same \
             diagram does; a bare id alongside a labelled one is rejected.\n\n\
             Showing an existing view id replaces it and brings it to the front; several named views \
             coexist. The flipchart dies with the session."
        )
    );
}

#[test]
fn clear_lleva_la_descripcion_literal_del_5_3() {
    let mut sesion = Sesion::abierta();

    let clear = herramienta(&sesion.herramientas(), "clear").clone();

    assert_eq!(
        clear["description"],
        json!("Remove one view from the flipchart, or all of them. Does not close the window.")
    );
}

#[test]
fn show_pide_view_id_y_diagram_y_los_dos_son_obligatorios() {
    let mut sesion = Sesion::abierta();

    let esquema = herramienta(&sesion.herramientas(), "show")["inputSchema"].clone();

    let mut obligatorios: Vec<&str> = esquema["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    obligatorios.sort();
    assert_eq!(obligatorios, ["diagram", "view_id"]);
}

#[test]
fn el_view_id_de_show_se_describe_con_su_ejemplo() {
    let mut sesion = Sesion::abierta();

    let esquema = herramienta(&sesion.herramientas(), "show")["inputSchema"].clone();

    assert_eq!(
        esquema["properties"]["view_id"]["description"],
        json!(
            "Short human-readable name, shown to the user above the diagram - e.g. \
             \"Current dependencies\", not \"v1\". Reusing a name replaces that view."
        )
    );
}

#[test]
fn clear_no_pide_nada() {
    let mut sesion = Sesion::abierta();

    let esquema = herramienta(&sesion.herramientas(), "clear")["inputSchema"].clone();

    assert!(
        esquema["required"]
            .as_array()
            .map(|r| r.is_empty())
            .unwrap_or(true)
    );
}

#[test]
fn show_devuelve_el_acuse_con_recuento_y_vistas_vivas() {
    let mut sesion = Sesion::abierta();

    let resultado = sesion.llama(
        "show",
        json!({ "view_id": "actual", "diagram": "flowchart LR\n  A[Uno] --> B[Dos]\n" }),
    );

    assert_eq!(
        texto(&resultado),
        "Shown as view \"actual\" (2 nodes, 1 edge). Views on the flipchart: actual."
    );
}

#[test]
fn un_aviso_viaja_con_la_vista_dibujada_y_no_como_error() {
    let mut sesion = Sesion::abierta();

    let resultado = sesion.llama(
        "show",
        json!({
            "view_id": "actual",
            "diagram": "flowchart TB\n  classDef danger fill:#f00\n  A[Uno] --> B[Dos]\n"
        }),
    );

    assert_eq!(resultado["isError"], json!(false));
}

#[test]
fn los_avisos_llegan_detras_del_acuse_y_se_acumulan() {
    let mut sesion = Sesion::abierta();

    let resultado = sesion.llama(
        "show",
        json!({
            "view_id": "actual",
            "diagram": "flowchart TB\n  classDef danger fill:#f00\n  A[Uno] --> B[Dos]\n"
        }),
    );

    assert_eq!(
        texto(&resultado),
        "Shown as view \"actual\" (2 nodes, 1 edge). Views on the flipchart: actual.\n\
         Note: style directives (classDef, class, style, linkStyle) and click links were \
         dropped — the flipchart decides how views look. The view was drawn.\n\
         Note: the flipchart lays diagrams out left to right; the direction in your source \
         was ignored. The view was drawn."
    );
}

#[test]
fn el_estado_de_la_pizarra_sobrevive_entre_llamadas() {
    let mut sesion = Sesion::abierta();
    let diagrama = json!("flowchart TD\n  A[Uno] --> B[Dos]\n");
    sesion.llama("show", json!({ "view_id": "actual", "diagram": diagrama }));
    sesion.llama(
        "show",
        json!({ "view_id": "propuesto", "diagram": diagrama }),
    );

    let resultado = sesion.llama("clear", json!({ "view_id": "actual" }));

    assert_eq!(
        texto(&resultado),
        "Cleared view \"actual\". Views on the flipchart: propuesto."
    );
}

#[test]
fn una_entrada_invalida_vuelve_marcada_como_error_de_herramienta() {
    let mut sesion = Sesion::abierta();

    let resultado = sesion.llama(
        "show",
        json!({ "view_id": "", "diagram": "flowchart TD\n A\n" }),
    );

    assert_eq!(resultado["isError"], json!(true));
}

#[test]
fn un_nodo_que_el_agente_no_declaro_vuelve_dentro_del_resultado_y_no_como_error_de_transporte() {
    let mut sesion = Sesion::abierta();

    let resultado = sesion.llama(
        "show",
        json!({ "view_id": "propuesto", "diagram": "flowchart TD\n  API[API Layer] --> Db\n" }),
    );

    assert_eq!(resultado["isError"], json!(true));
}

#[test]
fn el_rechazo_dice_que_no_se_dibujo_y_que_la_vista_sigue_como_estaba() {
    let mut sesion = Sesion::abierta();

    let resultado = sesion.llama(
        "show",
        json!({ "view_id": "propuesto", "diagram": "flowchart TD\n  API[API Layer] --> Db\n" }),
    );

    assert_eq!(
        texto(&resultado),
        "Rejected: nothing was drawn; view \"propuesto\" is unchanged.\n\
         1 node appears in the drawing that you did not declare.\n  \
         \"Db\"  line 2  — only used in a relation\n\
         Declare every node you name, and rewrite any line the renderer turned into one."
    );
}

#[test]
fn un_rechazo_no_toca_la_vista_que_ya_estaba_en_pantalla() {
    let mut sesion = Sesion::abierta();
    sesion.llama(
        "show",
        json!({ "view_id": "propuesto", "diagram": "flowchart TD\n  A[Uno] --> B[Dos]\n" }),
    );

    sesion.llama(
        "show",
        json!({ "view_id": "propuesto", "diagram": "flowchart TD\n  API[API Layer] --> Db\n" }),
    );

    assert_eq!(
        texto(&sesion.llama("clear", json!({ "view_id": "propuesto" }))),
        "Cleared view \"propuesto\". No views."
    );
}
