# Glossary

[Italiano](../glossary.md)

- **AXL Compact Source:** the canonical opcode-based source stream, optimized for agents and tokens.
- **AXL legacy frontend:** temporary keyword-based syntax for migration/debugging.
- **AX-IR:** AXL's family of typed, versioned intermediate representations.
- **AX-HIR:** the future high-level IR, close to AXL semantics.
- **AX-MIR:** the future lowered IR for VMs, optimization, and code generation.
- **AM:** the provider-agnostic, scoped, persistent memory module.
- **Agent:** an executable principal with an explicit scope and capabilities.
- **Workflow:** a composition of agents or other workflows.
- **Tool:** a host-implemented capability invokable through `call`.
- **Capability:** limited authorization to perform a class of effects.
- **Approval:** pre-effect consent required by a policy.
- **Audit:** a record of capability/approval decisions and outcomes.
- **Reference runtime:** the Python implementation used to establish semantics.
- **Rust runtime:** the target implementation for VM, native, WASM, and platforms.
- **Memory scope:** a host-controlled boundary that isolates persistent records.
- **Budget:** a limit applied to steps, values, output, tools, memory, or depth.
- **Provider:** an external implementation of a model, memory, database, or service; it is not part of the grammar.
