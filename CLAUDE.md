## Agent skills

### Issue tracker

Issues live as GitHub issues in `javierponferradalopez/ai-render`, via the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Domain docs

Single-context — `CONTEXT.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Construcción

Un solo crate binario `flipchart` en la raíz. La toolchain está pinchada en
`rust-toolchain.toml` y `rustup` la instala sola al entrar al repo.

| Comando | Qué hace |
|---|---|
| `make verify` | Formato, lints y tests. **Es la puerta**: lo mismo que corre la CI. |
| `make fmt` | `cargo fmt --check` |
| `make lint` | `cargo clippy --all-targets -- -D warnings` |
| `make test` | `cargo test` |
| `make build` | `cargo build --release` |

`flipchart check <diagram.mmd>...` corre la tubería sobre ficheros `.mmd` e
imprime el desenlace y el texto que recibiría el agente, **sin abrir ventana**.
Es el instrumento con el que se mide el Límite honesto.

Los lints viven en `[lints]` de `Cargo.toml`, no en banderas sueltas, para que
apliquen igual desde el editor. Dos importan más que el resto: `unused` en `deny`
—andamiaje que nadie lee es un error, no un aviso— y `clippy::print_stdout`,
porque **stdout es el transporte MCP** y un `println!` perdido corrompe el
protocolo. Para diagnóstico, stderr.

## Publicación

El release lo dispara un tag `v*` y lo corre `.github/workflows/publicacion.yml`. Sus dos
piezas con reglas propias son guiones probados, no pasos del YAML:

| Guión | Qué hace |
|---|---|
| `publicacion/empaqueta.sh <tag> <binario> <destino>` | Monta la caja —los cuatro ficheros, uno a uno— y la empaqueta con Info-ZIP. Escribe la ruta del zip en stdout. |
| `publicacion/catalogo.sh <tag> <zip> [repo]` | Genera el `marketplace.json` en stdout, con `version`, `url` y `sha256` sacados del tag y del propio zip. |

**El `marketplace.json` no se edita a mano nunca.** Subir de versión es tocar `Cargo.toml` y
`publicacion/caja/.claude-plugin/plugin.json`, y que el tag diga lo mismo: los dos guiones se
niegan si las tres no coinciden, y `tests/caja.rs` lo cuenta ya en `make verify` en vez de
esperar al release.
