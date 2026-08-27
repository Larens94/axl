use axl_compiler::compile_source;

const EXAMPLES: [(&str, &str); 8] = [
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
    (
        "open-dataview",
        include_str!("../../../examples/blocks/05-open-dataview.axl"),
    ),
    (
        "instance-override",
        include_str!("../../../examples/blocks/06-instance-override.axl"),
    ),
    (
        "software-foundation",
        include_str!("../../../examples/catalog/software-foundation.axl"),
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

#[test]
fn documented_invalid_examples_report_stable_codes() {
    let cases = [
        (
            "AXL-O401",
            include_str!("../../../examples/invalid/closed-blueprint.axl"),
        ),
        (
            "AXL-V403",
            include_str!("../../../examples/invalid/wrong-parameter.axl"),
        ),
        (
            "AXL-I605",
            include_str!("../../../examples/invalid/instance-overrides.axl"),
        ),
        (
            "AXL-I607",
            include_str!("../../../examples/invalid/instance-overrides.axl"),
        ),
        (
            "AXL-P405",
            include_str!("../../../examples/invalid/instance-overrides.axl"),
        ),
    ];

    for (code, source) in cases {
        let diagnostics = compile_source(source).expect_err("example must be rejected");
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.code == code),
            "missing {code}: {diagnostics:#?}"
        );
    }
}
