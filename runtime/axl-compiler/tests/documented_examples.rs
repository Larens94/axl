use axl_compiler::compile_source;

const EXAMPLES: [(&str, &str); 5] = [
    (
        "store",
        include_str!("../../../examples/blocks/01-store.axl"),
    ),
    (
        "ui-slot",
        include_str!("../../../examples/blocks/02-ui-slot.axl"),
    ),
    ("hook", include_str!("../../../examples/blocks/03-hook.axl")),
    (
        "agent",
        include_str!("../../../examples/blocks/04-agent.axl"),
    ),
    ("crm", include_str!("../../../examples/next/crm.axl")),
];

#[test]
fn every_documented_example_compiles_and_round_trips() {
    for (name, source) in EXAMPLES {
        let compiled = compile_source(source)
            .unwrap_or_else(|diagnostics| panic!("{name} failed: {diagnostics:#?}"));

        assert_eq!(compiled.graph.schema, "ax-ir/4.0", "{name}");
        let decoded = axl_compiler::next::packed::decode(&compiled.matrix)
            .unwrap_or_else(|error| panic!("{name} packed IR failed: {error}"));
        assert_eq!(decoded, compiled.graph, "{name}");
    }
}
