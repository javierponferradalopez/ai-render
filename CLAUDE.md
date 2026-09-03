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

Los lints viven en `[lints]` de `Cargo.toml`, no en banderas sueltas, para que
apliquen igual desde el editor. Dos importan más que el resto: `unused` en `deny`
—andamiaje que nadie lee es un error, no un aviso— y `clippy::print_stdout`,
porque **stdout es el transporte MCP** y un `println!` perdido corrompe el
protocolo. Para diagnóstico, stderr.
