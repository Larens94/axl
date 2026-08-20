# Security and Capabilities

[Italiano](../security.md)

## Threat model

The following are considered untrusted:

- AXL source;
- AX-IR JSON documents;
- model and tool output;
- content retrieved from memory;
- input from networks or users.

The following are considered trusted infrastructure in the reference implementation:

- the host process;
- explicitly loaded Python plugins;
- approval callbacks;
- memory adapters configured by the host.

## Fundamental rules

1. **Deny by default:** an unregistered tool cannot be executed.
2. **Least privilege:** an agent may use only the tools listed in `uses`.
3. **Pre-effect approval:** approval occurs before the handler runs.
4. **Fail closed:** only `approved is True` authorizes an action.
5. **Validation before effects:** AX-IR is validated before execution.
6. **Host-owned scopes:** the source cannot arbitrarily select a memory scope.
7. **Budgets:** loops, output, values, tools, and memory are limited.
8. **Secrets outside the language:** credentials must never appear in source, IR, output, or audit logs.
9. **Module root:** imports accept only top-level, relative `.axl` paths confined to the authorized directory; absolute paths and `..` are rejected.
10. **Import budgets:** depth, module count, and aggregate source bytes have fail-closed limits.
11. **Value budgets:** bytes, nodes, and collection depth are also checked for tool results and memory; SQLite performs a preflight check on persisted JSON before materialization.
12. **Canonical output:** the budget uses the value serialization emitted by the CLI; the transport line delimiter is excluded.

## Boundaries not covered

The reference implementation does not yet provide:

- sandboxing of Python plugin code;
- preemption of blocking handlers;
- OS/container isolation;
- kernel-enforced CPU/RAM limits;
- package signing;
- build attestation;
- OS-level capability-filtered networking.

These controls must be added by the deployment host and, later, by the Rust runtime.

## Target model

The Rust runtime must associate every effect with an unforgeable capability:

```text
agent principal
  + capability
  + resource scope
  + policy/version
  + budget/expiration
  + optional approval
  = authorized effect
```

Files, networks, databases, shells, GPUs, models, and memory will be distinct capabilities. Capabilities must not be represented as strings that programs can construct freely.

## Errors and audit

Errors must remain explicit and machine-readable. Audit logs record decisions, not secrets. In production, every approval should be tied to a run, principal, argument hash, policy, timestamp, and expiration.
