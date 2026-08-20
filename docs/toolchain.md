# Toolchain e utilizzo

## Requisiti correnti

- Python 3.11 o superiore;
- package `axl-lang` installabile con pip;
- SQLite incluso in Python per la memoria persistente.

Rust/Cargo non sono ancora richiesti dalla reference implementation.

## Installazione

```bash
python3 -m pip install .
axl --help
```

Per sviluppo:

```bash
python3 -m pip install --no-deps -e .
```

## Esecuzione sorgente canonico

```bash
axl run examples/compact.axl
```

## Canonicalizzazione

```bash
axl pack legacy.axl -o compact.axl
```

`pack` accetta il frontend legacy o compact, esegue validazione e type-check e produce Compact Source 2 normalizzato su una riga.

## Esecuzione sorgente con capability host

```bash
axl run examples/functions.axl
```

Con memoria e plugin:

```bash
axl run examples/agent_workflow.axl \
  --plugin examples.demo_tools \
  --approve-tool publish \
  --memory .axl-memory.sqlite \
  --scope project:demo
```

## Compilare ed eseguire AX-IR

```bash
axl compile examples/functions.axl -o functions.axlir.json
axl exec functions.axlir.json
```

`compile` risolve moduli, valida e type-checka prima di produrre AX-IR. `exec` valida nuovamente il documento prima degli effetti.

La CLI usa come **module root** la directory del file di ingresso. Gli import devono essere relativi, top-level, con estensione `.axl`, senza `..`, e restare entro tale root. L'API Python consente di restringere o definire esplicitamente il confine:

```python
from axl import compile_file

program = compile_file("src/app.axl", module_root="src")
```

Il resolver limita inoltre profondità, numero di moduli e byte sorgente aggregati.

## Budget CLI

Sono disponibili:

```text
--max-steps
--max-output-bytes
--max-value-bytes
--max-tool-calls
--max-memory-ops
--max-function-depth
```

## Test e qualità

```bash
python3 -m unittest discover -s tests -v
python3 -m ruff check .
python3 -m ruff format --check .
python3 -m compileall -q axl tests examples
python3 -m json.tool schema/axl-ir-1.0.schema.json >/dev/null
python3 -m json.tool schema/axl-ir-1.1.schema.json >/dev/null
git diff --check
```

## Toolchain target

La futura CLI unificata dovrà offrire comandi equivalenti a:

```text
axl new
axl check
axl build
axl run
axl test
axl fmt
axl doc
axl package
axl lsp
```

Il compilatore Rust dovrà supportare inizialmente interprete/VM e WASM, poi native AOT. Il corpus di test corrente fungerà da base della suite di conformità cross-runtime.
