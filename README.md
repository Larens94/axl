# AX / AXL — Agent eXecution

[Italiano](README.md) · [English](README.en.md)

**AX** è l'ecosistema Agent eXecution. **AXL** è il core eseguibile agent-native corrente: il sorgente canonico usa stream a singola riga, opcode numerici ed espressioni RPN, senza indentazione e con minimo overhead di token.

**Sito:** [larens94.github.io/axl](https://larens94.github.io/axl/) · **Documentazione:** [`docs/`](docs/README.md) · **Specifica:** [`SPEC.md`](SPEC.md)

```text
AXL Compact Source 2 → parser/type-checker → AX-IR/HIR/MIR
                                           → Rust/native
                                           → VM
                                           → WASM
                                           → bridge piattaforma
```

## Sorgente canonico

```axl
2;10|x|#2,#3,#4,*,+|i;12|$x
```

Il programma calcola `2 + 3 * 4`, assegna il risultato a `x: int` ed emette `14`.

- `2` — versione sorgente;
- `;` — frame;
- `|` — campi;
- `,` — token espressione RPN;
- `10` — binding;
- `12` — emit;
- `#` — integer;
- `$` — variabile;
- `99` — chiusura blocco.

Nessuna riga o indentazione è necessaria.

## Agente compatto

```axl
2;50|r|search;10|x|"AXL",!search/1;20|finding|$x|95|3600|r;12|$x;99;51|w;52|r;99;52|w
```

Questo dichiara un agente `r` con grant `search`, chiama la capability, salva il risultato in AM, emette l'output, crea il workflow `w` e lo esegue.

## Obiettivo

AXL dovrà permettere agli agenti di costruire qualsiasi software:

- backend, API, servizi e database;
- frontend web e WebAssembly;
- app desktop e mobile native;
- CLI, sistemi e automazioni;
- grafica, GPU, audio e giochi;
- agenti, workflow, task ed eventi.

Rust è il primo runtime/backend di basso livello, **non un vincolo**. Bridge versionati consentiranno Rust/C ABI, WASI, DOM, GPU, mobile, OS e futuri target mantenendo invariata la semantica AXL.

## Disponibile

- Compact Source 2 e writer canonico;
- parser deterministico ed espressioni RPN;
- `axl pack` per migrare il frontend legacy;
- tipi `int`, `string`, `bool` e `list<T>` omogenee;
- funzioni, parametri, ritorni, moduli e namespace;
- condizioni e cicli limitati;
- agenti, workflow e tool grants;
- AM con scope, in memoria/SQLite, metadati e TTL;
- policy con negazione predefinita, approvazione a chiusura sicura e audit;
- budget multidimensionali;
- AX-IR JSON 1.0/1.1/1.2;
- CLI `run`, `pack`, `compile`, `exec`.

## Installazione e uso

```bash
cargo build --workspace
cargo run -p axl-cli -- run examples/compact.axl
cargo run -p axl-cli -- compile examples/compact.axl -o program.axlir.json
cargo run -p axl-cli -- exec program.axlir.json
```

Conversione del vecchio frontend leggibile:

```bash
cargo run -p axl-cli -- pack legacy.axl -o canonical.axl
```

Il formato keyword-based resta temporaneamente supportato per migrazione/debug. Non è più la sintassi primaria.

## Tool host

```rust
use axl_core::{Tool, Value};

fn tools() -> Vec<Tool> {
    vec![
        Tool::new("search", Box::new(|args| Ok(Value::String("result".into()))))
            .with_effect("read"),
        Tool::new("publish", Box::new(|args| Ok(Value::Bool(true))))
            .with_effect("write")
            .with_approval(true),
    ]
}
```

I plugin sono infrastruttura host fidata. AXL limita capability, scope, policy, approvazioni e budget; il codice plugin arbitrario richiede sandbox esterna.

## Qualità

```bash
cargo test --workspace
cargo clippy --workspace
```

AXL è distribuito con licenza [Apache-2.0](LICENSE).

## Toolchain Rust

Il workspace Rust contiene l'intero runtime AXL:

- Parser compatto (Source 2 e 3) e legacy keyword-based
- Type-checker con supporto `int`, `string`, `bool`, `list<T>`, `map<K,V>`
- Interpreter con memory store, policy, tool grants e budget
- Serializzazione AX-IR JSON 1.0/1.1/1.2
- Renderer web con hot-reload server

```bash
source "$HOME/.cargo/env"
cargo test --workspace
cargo run -p axl-cli -- serve examples/streaming_home.axl
```

Aprire quindi `http://localhost:8000`. Il comando compila, serve e ricompila il
sorgente quando cambia. Il solo sorgente applicativo è
`examples/streaming_home.axl`; HTML, CSS e JavaScript sono generati da Rust.
