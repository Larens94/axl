# AXL — Agent eXecution Language

AXL is an in-development general-purpose, agent-native programming language with a deterministic Python reference runtime.

**Website:** [larens94.github.io/axl](https://larens94.github.io/axl/) · **Documentation:** [`docs/`](docs/README.md)

```text
AXL source → parser/type-checker → validated AX-IR 1.1 → budgeted runtime
                                      ├─ scoped memory adapters
                                      ├─ explicit tool plugins
                                      ├─ policy + approvals
                                      └─ audit events
```

## V1 capabilities

- typed values: `string`, `integer`, `boolean`;
- typed functions, parameters, returns, calls and isolated local scopes;
- relative modules with explicit aliases, namespace isolation and cycle detection;
- expressions, comparisons, `if/else`, bounded `while`;
- named agents with explicit tool grants;
- sequential named workflows;
- typed, scoped memory with confidence, source, TTL, versioning and `forget`;
- in-memory and SQLite memory adapters;
- deny-by-default tools, pre-effect approval and audit events;
- instruction/expression, function-depth, intermediate-value, output, tool-call and memory-operation budgets;
- validated, versioned JSON IR with `compile` and `exec`;
- explicit Python plugin boundary;
- static validation for declarations, references and workflow cycles.

## Install

```bash
python3 -m pip install .
axl --help
```

## Run source

```bash
axl run examples/agent_workflow.axl \
  --plugin examples.demo_tools \
  --approve-tool publish \
  --memory .axl-memory.sqlite \
  --scope project:demo
```

Expected output:

```text
research:AXL
published:research:AXL
```

## Compile and execute IR

```bash
axl compile examples/agent_workflow.axl -o program.axlir.json
axl exec program.axlir.json --plugin examples.demo_tools --approve-tool publish
```

## Modules and typed functions

```axl
import math from "math.axl"

fn describe(value: int) -> string
    if value >= 10
        return "large"
    else
        return "small"
    end
end

let total: int = math.add(7, 8)
emit describe(total)
```

Imports are resolved relative to the importing file. Imported modules export function declarations only; top-level effects are rejected.

## Example language

```axl
agent researcher uses search
    let finding = call search("AXL")
    memory finding = finding meta confidence=95 ttl=3600 source=researcher
    emit finding
end

agent publisher uses publish
    let finding = recall finding
    emit call publish(finding)
end

workflow release
    run researcher
    run publisher
end

run release
```

## Tool plugin

```python
from axl import Tool


def tools():
    return [
        Tool("search", search, effect="read"),
        Tool("publish", publish, effect="write", approval=True),
    ]
```

Plugins are trusted host code. AXL restricts which registered capability an agent may invoke; it does not sandbox arbitrary Python plugin internals.

## Tests

```bash
python3 -m unittest discover -s tests -v
```

## Documentation

Start from [`docs/README.md`](docs/README.md) for the complete project guide:

- language vision and current status;
- stack architecture and Rust/native/WASM target;
- syntax and language guide;
- agents, workflows, tools and AM memory;
- AX-IR compatibility and security model;
- toolchain and milestone roadmap.

## Open source

AXL is released under the [Apache License 2.0](LICENSE).

## V1 boundaries

The reference runtime intentionally excludes distributed scheduling, parallel DAGs, dynamic agent spawning, embedded LLM providers, suspended approvals, and native sandboxing. AX-IR is the versioned integration boundary; future Rust/WASM runtimes must preserve its validation, budget, memory and policy semantics.

See [SPEC.md](SPEC.md), [AX-IR 1.0](schema/axl-ir-1.0.schema.json), [AX-IR 1.1](schema/axl-ir-1.1.schema.json), [documentation](docs/README.md), and [CHANGELOG.md](CHANGELOG.md).
