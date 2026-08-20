# Agents, Workflows, Capabilities, and AM

[Italiano](../agent-runtime.md)

## Deterministic, not probabilistic

Agent primitives belong to the language, but their parsing and control are deterministic. AI models, search, filesystems, networks, and services are typed runtime capabilities.

## Agent

An agent is a principal with a name, an isolated frame, explicit grants, and an executable body.

```axl
2;50|researcher|search;10|finding|"AXL",!search/1;20|finding|$finding|90|-|researcher;12|$finding;99
```

- `50|researcher|search` opens the agent and grants only `search`;
- `!search/1` consumes one argument and invokes the capability;
- `99` closes the body.

A tool registered by the host but not granted to the agent is denied.

## Workflow

```axl
2;51|release;52|researcher;52|publisher;99;52|release
```

`51` opens the workflow, `52` executes a runnable, and `99` closes it. The current runtime is sequential and rejects cycles. DAGs, parallelism, retries, and checkpoints are on the roadmap.

## Host capabilities

A capability declares a name, ABI, input/output, effect, target, and policy. Today, the Python bridge uses `Tool`:

```python
from axl import Tool


def tools():
    return [
        Tool("search", search, effect="read"),
        Tool("publish", publish, effect="write", approval=True),
    ]
```

Future Rust/C/WASI/DOM/GPU bridges will implement the same model. Capabilities do not become vendor-specific keywords in the source language.

## Fail-closed approval

Only the exact Boolean value `True` grants authorization. Truthy strings, numbers, exceptions, or a missing provider result in denial.

## Audit

Events: `approval_required`, `approved`, `denied`, `executed`, `failed`. Secrets and credentials must be resolved inside the bridge and must not appear in source, IR, output, or audit logs.

## AM — memory

AM is the memory module, not the name of the language.

```axl
2;20|finding|"result"|95|3600|researcher;12|@finding;21|finding
```

Properties:

- provider-agnostic protocol;
- in-memory and SQLite adapters;
- host-controlled scopes;
- typed values;
- confidence, source, version, timestamp, and TTL;
- explicit deletion.

There is no automatic promotion between scopes. Vector, graph, or cloud backends must preserve the AM contract.

## Budgets

Limits apply to steps/expressions, call depth, values, output, tool calls, and memory. Blocking or untrusted capabilities require host isolation, timeouts, and cancellation.
