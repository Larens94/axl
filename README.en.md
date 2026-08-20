**Languages:** [Italiano](README.md) · [English](README.en.md)

# AX / AXL — Agent eXecution

**AX** is the Agent eXecution ecosystem. **AXL** is the current executable agent-native core: the canonical source uses single-line streams, numeric opcodes, and RPN expressions, without indentation and with minimal token overhead.

**Website:** [larens94.github.io/axl/en/](https://larens94.github.io/axl/en/) · **Documentation:** [`docs/en/`](docs/en/README.md) · **Specification:** [`SPEC.en.md`](SPEC.en.md)

```text
AXL Compact Source 2 → parser/type-checker → AX-IR/HIR/MIR
                                           → Rust/native
                                           → VM
                                           → WASM
                                           → platform bridge
```

## Canonical source

```axl
2;10|x|#2,#3,#4,*,+|i;12|$x
```

The program computes `2 + 3 * 4`, assigns the result to `x: int`, and emits `14`.

- `2` — source version;
- `;` — frame;
- `|` — fields;
- `,` — RPN expression token;
- `10` — binding;
- `12` — emit;
- `#` — integer;
- `$` — variable;
- `99` — end block.

No line breaks or indentation are required.

## Compact agent

```axl
2;50|r|search;10|x|"AXL",!search/1;20|finding|$x|95|3600|r;12|$x;99;51|w;52|r;99;52|w
```

This declares an agent `r` with a `search` grant, calls the capability, stores the result in AM, emits the output, creates the workflow `w`, and runs it.

## Goal

AXL will enable agents to build any kind of software:

- backends, APIs, services, and databases;
- web frontends and WebAssembly;
- native desktop and mobile apps;
- CLIs, systems, and automation;
- graphics, GPU, audio, and games;
- agents, workflows, tasks, and events.

Rust is the first low-level runtime/backend, **not a constraint**. Versioned bridges will support Rust/C ABI, WASI, DOM, GPU, mobile, OS, and future targets while keeping AXL semantics unchanged.

## Available

- Compact Source 2 and canonical writer;
- deterministic parser and RPN expressions;
- `axl pack` to migrate the legacy frontend;
- `int`, `string`, `bool`, and homogeneous `list<T>` types;
- functions, parameters, returns, modules, and namespaces;
- conditions and bounded loops;
- agents, workflows, and tool grants;
- scoped AM, in memory/SQLite, with metadata and TTL;
- deny-by-default policy, fail-closed approval, and auditing;
- multidimensional budgets;
- AX-IR JSON 1.0/1.1/1.2;
- CLI commands `run`, `pack`, `compile`, `exec`.

## Installation and usage

```bash
python3 -m pip install .
axl run examples/compact.axl
axl compile examples/compact.axl -o program.axlir.json
axl exec program.axlir.json
```

Converting the old readable frontend:

```bash
axl pack legacy.axl -o canonical.axl
```

The keyword-based format remains temporarily supported for migration/debugging. It is no longer the primary syntax.

## Tool host

```python
from axl import Tool


def tools():
    return [
        Tool("search", search, effect="read"),
        Tool("publish", publish, effect="write", approval=True),
    ]
```

Plugins are trusted host infrastructure. AXL restricts capabilities, scopes, policies, approvals, and budgets; arbitrary plugin code requires an external sandbox.

## Quality

```bash
python3 -m unittest discover -s tests -v
python3 -m ruff check .
python3 -m ruff format --check .
```

AXL is distributed under the [Apache-2.0](LICENSE) license.
