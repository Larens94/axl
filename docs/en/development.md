# Development and Contribution

[Italiano](../development.md)

## Philosophy

AXL evolves through complete vertical slices:

```text
source → parser → IR → validation/type-check → runtime → observable output
```

A grammar change is not complete without consistent IR, semantics, diagnostics, tests, examples, and documentation.

## TDD Workflow

For each behavior:

1. write a focused test;
2. run it and verify the expected failure;
3. implement the minimum necessary;
4. rerun the focused test and the full suite;
5. refactor while keeping the tests green.

## Local Gates

```bash
python3 -m unittest discover -s tests -q
python3 -m ruff check .
python3 -m ruff format --check .
python3 -m compileall -q axl tests examples
python3 -m json.tool schema/axl-ir-1.0.schema.json >/dev/null
python3 -m json.tool schema/axl-ir-1.1.schema.json >/dev/null
python3 -m json.tool schema/axl-ir-1.2.schema.json >/dev/null
git diff --check
```

Also verify at least one source program and the same program through `compile`→`exec`.

## Language Changes

A proposal must specify:

- problem and use cases;
- proposed syntax;
- type rules;
- HIR/IR nodes;
- runtime semantics and effects;
- diagnostics;
- security and capability impact;
- compatibility and migration;
- conformance tests.

## AX-IR Compatibility

Do not modify a published schema. An incompatible change requires:

1. an AX-IR version increment;
2. a new schema file;
3. a decoder for the new version;
4. a legacy upgrade path or an explicit error;
5. round-trip and compatibility tests;
6. specification and changelog updates.

## Layer Separation

- the parser does not execute effects;
- the IR does not contain specific clients/providers;
- the runtime does not interpret source syntax;
- plugins implement host capabilities;
- secrets and credentials do not enter source, IR, or audit logs.

## Security Reports

Do not publish exploitable vulnerabilities or credentials in public issues. Contact the repository maintainer privately until a formal security reporting process is established.
