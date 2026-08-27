use axl_compiler::compile_source;

const EXAMPLES: [(&str, &str); 9] = [
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
    (
        "cashflow-core",
        include_str!("../../../examples/apps/cashflow-core.axl"),
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
        let formatted = compile_source(&compiled.source).unwrap_or_else(|diagnostics| {
            panic!("{name} formatted source failed: {diagnostics:#?}")
        });
        assert_eq!(formatted.graph, compiled.graph, "{name} formatted graph");
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
        (
            "AXL-X803",
            include_str!("../../../examples/invalid/flow-types.axl"),
        ),
        (
            "AXL-X806",
            include_str!("../../../examples/invalid/flow-types.axl"),
        ),
        (
            "AXL-X816",
            include_str!("../../../examples/invalid/flow-calls.axl"),
        ),
        (
            "AXL-X817",
            include_str!("../../../examples/invalid/flow-calls.axl"),
        ),
        (
            "AXL-X818",
            include_str!("../../../examples/invalid/flow-calls.axl"),
        ),
        (
            "AXL-X819",
            include_str!("../../../examples/invalid/flow-calls.axl"),
        ),
        (
            "AXL-X820",
            include_str!("../../../examples/invalid/flow-calls.axl"),
        ),
        (
            "AXL-X821",
            include_str!("../../../examples/invalid/flow-calls.axl"),
        ),
        (
            "AXL-X831",
            include_str!("../../../examples/invalid/flow-records.axl"),
        ),
        (
            "AXL-X832",
            include_str!("../../../examples/invalid/flow-records.axl"),
        ),
        (
            "AXL-X833",
            include_str!("../../../examples/invalid/flow-records.axl"),
        ),
        (
            "AXL-X834",
            include_str!("../../../examples/invalid/flow-records.axl"),
        ),
        (
            "AXL-X835",
            include_str!("../../../examples/invalid/flow-records.axl"),
        ),
        (
            "AXL-N805",
            include_str!("../../../examples/invalid/flow-folds.axl"),
        ),
        (
            "AXL-X841",
            include_str!("../../../examples/invalid/flow-folds.axl"),
        ),
        (
            "AXL-X842",
            include_str!("../../../examples/invalid/flow-folds.axl"),
        ),
        (
            "AXL-X843",
            include_str!("../../../examples/invalid/flow-folds.axl"),
        ),
        (
            "AXL-X851",
            include_str!("../../../examples/invalid/flow-runs.axl"),
        ),
        (
            "AXL-X852",
            include_str!("../../../examples/invalid/flow-runs.axl"),
        ),
        (
            "AXL-X853",
            include_str!("../../../examples/invalid/flow-runs.axl"),
        ),
        (
            "AXL-X854",
            include_str!("../../../examples/invalid/flow-runs.axl"),
        ),
        (
            "AXL-X855",
            include_str!("../../../examples/invalid/flow-runs.axl"),
        ),
        (
            "AXL-X856",
            include_str!("../../../examples/invalid/flow-runs.axl"),
        ),
        (
            "AXL-X861",
            include_str!("../../../examples/invalid/flow-matches.axl"),
        ),
        (
            "AXL-X862",
            include_str!("../../../examples/invalid/flow-matches.axl"),
        ),
        (
            "AXL-X863",
            include_str!("../../../examples/invalid/flow-matches.axl"),
        ),
        (
            "AXL-X864",
            include_str!("../../../examples/invalid/flow-matches.axl"),
        ),
        (
            "AXL-X865",
            include_str!("../../../examples/invalid/flow-matches.axl"),
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

#[test]
fn documented_cashflow_core_executes() {
    let source = include_str!("../../../examples/apps/cashflow-core.axl");
    let graph = compile_source(source).unwrap().graph;
    let movement = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-valid.json"
    ))
    .unwrap();
    let balance =
        serde_json::from_str(include_str!("../../../examples/apps/inputs/balance.json")).unwrap();

    let validated =
        axl_compiler::next::runtime::evaluate_flow(&graph, "ValidateMovement", movement).unwrap();
    assert_eq!(validated["ok"]["kind"], "income");

    let calculated =
        axl_compiler::next::runtime::evaluate_flow(&graph, "CalculateBalance", balance).unwrap();
    assert_eq!(calculated, 80000);

    let movement = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-valid.json"
    ))
    .unwrap();
    let view =
        axl_compiler::next::runtime::evaluate_flow(&graph, "BuildMovementView", movement).unwrap();
    assert_eq!(view["direction"], "Entrata");
    assert_eq!(view["signed_amount"], 125000);

    let movements = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-batch.json"
    ))
    .unwrap();
    let ledger =
        axl_compiler::next::runtime::evaluate_flow(&graph, "CalculateLedgerBalance", movements)
            .unwrap();
    assert_eq!(ledger, 80000);

    for flow in ["StoreAndLoadMovement", "StoreAndLoadMovementSqlite"] {
        let movement = serde_json::from_str(include_str!(
            "../../../examples/apps/inputs/movement-valid.json"
        ))
        .unwrap();
        let stored = axl_compiler::next::runtime::evaluate_flow(&graph, flow, movement).unwrap();
        assert_eq!(stored["ok"]["id"], "movement-001", "{flow}");
    }

    let movement = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-valid.json"
    ))
    .unwrap();
    let composed =
        axl_compiler::next::runtime::evaluate_flow(&graph, "ValidateAndStoreMovement", movement)
            .unwrap();
    assert_eq!(composed["ok"]["id"], "movement-001");

    let invalid = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-invalid.json"
    ))
    .unwrap();
    let rejected =
        axl_compiler::next::runtime::evaluate_flow(&graph, "ValidateAndStoreMovement", invalid)
            .unwrap();
    assert_eq!(rejected["error"], "amount_must_be_positive");
}
