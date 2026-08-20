# AXL 1.1 development specification

## Execution model

AXL programs compile into validated, typed IR. Declarations are inert. `run <name>` invokes a declared agent or workflow sequentially. Runtime effects occur only through memory adapters and explicitly registered tools.

## Grammar

```ebnf
source       = { import }, program ;
import       = "import", id, "from", string ;
program      = { instruction } ;
instruction  = memory | forget | binding | return | emit | if | while |
               function | agent | workflow | run ;
memory       = "memory", id, "=", expression,
               [ "meta", metadata, { metadata } ] ;
metadata     = ("confidence" | "ttl" | "source"), "=", value ;
forget       = "forget", id ;
binding      = "let", id, [ ":", type ], "=", expression ;
return       = "return", expression ;
function     = "fn", id, "(", [ parameter, { ",", parameter } ], ")",
               "->", type, block, "end" ;
parameter    = id, ":", type ;
type         = "int" | "string" | "bool" ;
emit         = "emit", expression ;
if           = "if", expression, block, [ "else", block ], "end" ;
while        = "while", expression, block, "end" ;
agent        = "agent", id, [ "uses", id, { ",", id } ], block, "end" ;
workflow     = "workflow", id, block, "end" ;
run          = "run", id ;
expression   = primary, { operator, primary } ;
primary      = string | integer | boolean | id | "recall", id |
               id, "(", [ expression, { ",", expression } ], ")" |
               "call", id, "(", [ expression, { ",", expression } ], ")" |
               "(", expression, ")" ;
operator     = "+" | "-" | "*" | "/" | "==" | "!=" |
               ">" | "<" | ">=" | "<=" ;
```

Blank lines and full-line `#` comments are ignored. Reserved words cannot be identifiers.

Imports are compiler directives and do not survive into AX-IR. Paths resolve relative to the importing source. Aliases create qualified function namespaces. Duplicate aliases, import cycles and effectful module top levels are compile errors.

## Types and operators

Values are exactly strings, integers, or booleans. Python coercions are forbidden.

- `+`: integer addition or string concatenation with identical types;
- `-`, `*`, `/`, ordering: integers only;
- `/`: division by zero and fractional results are errors;
- equality: operands must have identical runtime types;
- conditions: booleans only;
- tool results outside the value set are runtime errors.

Functions declare typed parameters and one return type. Calls are checked before CLI execution; wrong arity, unknown functions, incompatible arguments, incompatible returns and missing returns are static errors. Every call receives an isolated local frame. Runtime call depth is bounded.

## Agents and workflows

An agent is a principal with a fixed set of tool grants and an isolated local-variable frame. A workflow is a sequential block that may run agents or workflows. Names are unique. Unknown references and recursive call cycles are rejected before effects.

A tool call succeeds only when:

1. the host explicitly registered the tool;
2. the active agent declared the tool in `uses`;
3. any required approval is granted before invocation;
4. runtime budgets permit the call;
5. the handler returns a valid AXL value.

Top-level calls are host-context calls and remain limited to explicitly registered tools. Production integrations should prefer agent-scoped calls.

## Memory

`MemoryStore` is provider-neutral. Runtime scope is host-provided and cannot be changed by source code. A memory record contains:

- scope and key;
- typed value;
- monotonically increasing version;
- confidence `0..100`;
- source identifier;
- update timestamp;
- optional expiry timestamp.

TTL expiry is enforced on read/inspection. `forget` is idempotent and confined to the active scope. SQLite upgrades legacy V0.3 tables on open.

Writes commit per memory instruction. A later program failure does not roll back earlier external effects; workflows requiring atomic domain behavior must expose one transactional host tool.

## Policy and audit

Tools declare an effect (`read`, `write`, `destructive`, or host-defined) and whether approval is required. Approval receives the exact tool name, typed arguments and effect. Missing or denied approval prevents execution. Events record approval-required, approved, denied, executed, or failed decisions.

CLI `--approve-tool NAME` is explicit non-interactive preapproval by name for the current process. Rich integrations should bind approval to run, principal, argument hash, policy version and expiry.

## Budgets

The runtime enforces positive limits for:

- instructions and expression evaluations;
- intermediate value bytes (UTF-8 for strings, magnitude bytes for integers);
- output UTF-8 bytes;
- tool calls;
- memory operations.

These budgets do not interrupt blocking trusted Python plugin code. Host deployments must execute untrusted or long-running tools in an isolated process/container with deadlines.

## JSON AX-IR 1.1

The envelope is:

```json
{"ir_version":"1.1","program":{"type":"Program","instructions":[]}}
```

The decoder rejects unknown versions/nodes/fields, invalid node placement, invalid literal types, invalid operators, malformed collections, unresolved references and cycles. It accepts AX-IR 1.0 through a tested legacy upgrade. Published schemas are immutable and versioned separately. Semantic validation is authoritative even when schema validation is not performed externally.

## Security boundaries

- source and JSON are untrusted inputs;
- decoded IR is validated before effects;
- tools are deny-by-default and agent-granted;
- memory scope is host-owned;
- plugin code is trusted infrastructure, not sandboxed AXL code;
- secrets should be resolved inside tools and never represented in AXL source, IR, output or audit arguments.
