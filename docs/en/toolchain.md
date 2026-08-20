# Toolchain and Usage

[Italiano](../toolchain.md)

## Current requirements

- Python 3.11 or later;
- the pip-installable `axl-lang` package;
- SQLite, included with Python, for persistent memory.

Rust/Cargo are not yet required by the reference implementation.

## Installation

```bash
python3 -m pip install .
axl --help
```

For development:

```bash
python3 -m pip install --no-deps -e .
```

## Running canonical source

```bash
axl run examples/compact.axl
```

## Canonicalization

```bash
axl pack legacy.axl -o compact.axl
```

`pack` accepts either the legacy or compact frontend, performs validation and type-checking, and produces normalized single-line Compact Source 2.

## Running source with host capabilities

```bash
axl run examples/functions.axl
```

With memory and plugins:

```bash
axl run examples/agent_workflow.axl \
  --plugin examples.demo_tools \
  --approve-tool publish \
  --memory .axl-memory.sqlite \
  --scope project:demo
```

## Compiling and executing AX-IR

```bash
axl compile examples/functions.axl -o functions.axlir.json
axl exec functions.axlir.json
```

`compile` resolves modules, validates, and type-checks before producing AX-IR. `exec` validates the document again before any effects occur.

The CLI uses the directory containing the input file as its **module root**. Imports must be relative, top-level, have an `.axl` extension, contain no `..`, and remain within that root. The Python API allows this boundary to be restricted or defined explicitly:

```python
from axl import compile_file

program = compile_file("src/app.axl", module_root="src")
```

The resolver also limits depth, module count, and aggregate source bytes.

## CLI budgets

The following options are available:

```text
--max-steps
--max-output-bytes
--max-value-bytes
--max-value-nodes
--max-value-depth
--max-tool-calls
--max-memory-ops
--max-function-depth
```

## Testing and quality

```bash
python3 -m unittest discover -s tests -v
python3 -m ruff check .
python3 -m ruff format --check .
python3 -m compileall -q axl tests examples
python3 -m json.tool schema/axl-ir-1.0.schema.json >/dev/null
python3 -m json.tool schema/axl-ir-1.1.schema.json >/dev/null
python3 -m json.tool schema/axl-ir-1.2.schema.json >/dev/null
git diff --check
```

## Target toolchain

The future unified CLI must provide commands equivalent to:

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

The Rust compiler must initially support an interpreter/VM and WASM, followed by native AOT. The current test corpus will serve as the foundation of the cross-runtime conformance suite.
