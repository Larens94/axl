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
            "AXL-P313",
            include_str!("../../../examples/invalid/provider-config-syntax.axl"),
        ),
        (
            "AXL-P913",
            include_str!("../../../examples/invalid/http-auth-syntax.axl"),
        ),
        (
            "AXL-P914",
            include_str!("../../../examples/invalid/http-auth-syntax.axl"),
        ),
        (
            "AXL-P915",
            include_str!("../../../examples/invalid/http-auth-syntax.axl"),
        ),
        (
            "AXL-P916",
            include_str!("../../../examples/invalid/http-auth-syntax.axl"),
        ),
        (
            "AXL-P917",
            include_str!("../../../examples/invalid/http-auth-syntax.axl"),
        ),
        (
            "AXL-H908",
            include_str!("../../../examples/invalid/http-auth.axl"),
        ),
        (
            "AXL-H909",
            include_str!("../../../examples/invalid/http-auth.axl"),
        ),
        (
            "AXL-H910",
            include_str!("../../../examples/invalid/http-auth.axl"),
        ),
        (
            "AXL-H911",
            include_str!("../../../examples/invalid/http-auth.axl"),
        ),
        (
            "AXL-H912",
            include_str!("../../../examples/invalid/http-auth.axl"),
        ),
        (
            "AXL-H913",
            include_str!("../../../examples/invalid/http-request-bindings.axl"),
        ),
        (
            "AXL-H914",
            include_str!("../../../examples/invalid/http-request-bindings.axl"),
        ),
        (
            "AXL-H915",
            include_str!("../../../examples/invalid/http-request-bindings.axl"),
        ),
        (
            "AXL-H916",
            include_str!("../../../examples/invalid/http-request-bindings.axl"),
        ),
        (
            "AXL-H917",
            include_str!("../../../examples/invalid/http-request-bindings.axl"),
        ),
        (
            "AXL-P918",
            include_str!("../../../examples/invalid/http-middleware-syntax.axl"),
        ),
        (
            "AXL-H918",
            include_str!("../../../examples/invalid/http-middleware.axl"),
        ),
        (
            "AXL-H919",
            include_str!("../../../examples/invalid/http-middleware.axl"),
        ),
        (
            "AXL-H920",
            include_str!("../../../examples/invalid/http-middleware.axl"),
        ),
        (
            "AXL-H921",
            include_str!("../../../examples/invalid/http-middleware.axl"),
        ),
        (
            "AXL-H922",
            include_str!("../../../examples/invalid/http-middleware.axl"),
        ),
        (
            "AXL-P920",
            include_str!("../../../examples/invalid/flow-events-syntax.axl"),
        ),
        (
            "AXL-E901",
            include_str!("../../../examples/invalid/flow-events.axl"),
        ),
        (
            "AXL-E902",
            include_str!("../../../examples/invalid/flow-events.axl"),
        ),
        (
            "AXL-E903",
            include_str!("../../../examples/invalid/flow-events.axl"),
        ),
        (
            "AXL-E904",
            include_str!("../../../examples/invalid/flow-events.axl"),
        ),
        (
            "AXL-E905",
            include_str!("../../../examples/invalid/flow-events.axl"),
        ),
        (
            "AXL-E906",
            include_str!("../../../examples/invalid/flow-events.axl"),
        ),
        (
            "AXL-P921",
            include_str!("../../../examples/invalid/flow-jobs-syntax.axl"),
        ),
        (
            "AXL-J901",
            include_str!("../../../examples/invalid/flow-jobs.axl"),
        ),
        (
            "AXL-J902",
            include_str!("../../../examples/invalid/flow-jobs.axl"),
        ),
        (
            "AXL-J903",
            include_str!("../../../examples/invalid/flow-jobs.axl"),
        ),
        (
            "AXL-J904",
            include_str!("../../../examples/invalid/flow-jobs.axl"),
        ),
        (
            "AXL-J905",
            include_str!("../../../examples/invalid/flow-jobs.axl"),
        ),
        (
            "AXL-J906",
            include_str!("../../../examples/invalid/flow-jobs.axl"),
        ),
        (
            "AXL-J907",
            include_str!("../../../examples/invalid/flow-jobs.axl"),
        ),
        (
            "AXL-J908",
            include_str!("../../../examples/invalid/flow-jobs.axl"),
        ),
        (
            "AXL-P314",
            include_str!("../../../examples/invalid/provider-config-syntax.axl"),
        ),
        (
            "AXL-N303",
            include_str!("../../../examples/invalid/provider-configs.axl"),
        ),
        (
            "AXL-N304",
            include_str!("../../../examples/invalid/provider-configs.axl"),
        ),
        (
            "AXL-V305",
            include_str!("../../../examples/invalid/provider-configs.axl"),
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
        (
            "AXL-X802",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-N806",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X871",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X872",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X873",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X874",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X875",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X876",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X877",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X878",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X879",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X881",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X882",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X883",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X884",
            include_str!("../../../examples/invalid/flow-transforms.axl"),
        ),
        (
            "AXL-X891",
            include_str!("../../../examples/invalid/flow-parallel.axl"),
        ),
        (
            "AXL-X892",
            include_str!("../../../examples/invalid/flow-parallel.axl"),
        ),
        (
            "AXL-X893",
            include_str!("../../../examples/invalid/flow-parallel.axl"),
        ),
        (
            "AXL-X894",
            include_str!("../../../examples/invalid/flow-parallel.axl"),
        ),
        (
            "AXL-X895",
            include_str!("../../../examples/invalid/flow-parallel.axl"),
        ),
        (
            "AXL-X901",
            include_str!("../../../examples/invalid/flow-attempts.axl"),
        ),
        (
            "AXL-X902",
            include_str!("../../../examples/invalid/flow-attempts.axl"),
        ),
        (
            "AXL-X903",
            include_str!("../../../examples/invalid/flow-attempts.axl"),
        ),
        (
            "AXL-X904",
            include_str!("../../../examples/invalid/flow-attempts.axl"),
        ),
        (
            "AXL-X905",
            include_str!("../../../examples/invalid/flow-attempts.axl"),
        ),
        (
            "AXL-X906",
            include_str!("../../../examples/invalid/flow-attempts.axl"),
        ),
        (
            "AXL-X907",
            include_str!("../../../examples/invalid/flow-attempts.axl"),
        ),
        (
            "AXL-X911",
            include_str!("../../../examples/invalid/flow-races.axl"),
        ),
        (
            "AXL-X912",
            include_str!("../../../examples/invalid/flow-races.axl"),
        ),
        (
            "AXL-X913",
            include_str!("../../../examples/invalid/flow-races.axl"),
        ),
        (
            "AXL-X914",
            include_str!("../../../examples/invalid/flow-races.axl"),
        ),
        (
            "AXL-X915",
            include_str!("../../../examples/invalid/flow-races.axl"),
        ),
        (
            "AXL-X916",
            include_str!("../../../examples/invalid/flow-races.axl"),
        ),
        (
            "AXL-H901",
            include_str!("../../../examples/invalid/http-routes.axl"),
        ),
        (
            "AXL-H902",
            include_str!("../../../examples/invalid/http-routes.axl"),
        ),
        (
            "AXL-H903",
            include_str!("../../../examples/invalid/http-routes.axl"),
        ),
        (
            "AXL-H904",
            include_str!("../../../examples/invalid/http-routes.axl"),
        ),
        (
            "AXL-H905",
            include_str!("../../../examples/invalid/http-routes.axl"),
        ),
        (
            "AXL-H906",
            include_str!("../../../examples/invalid/http-routes.axl"),
        ),
        (
            "AXL-H907",
            include_str!("../../../examples/invalid/http-routes.axl"),
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

    let movements = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-batch.json"
    ))
    .unwrap();
    let amounts =
        axl_compiler::next::runtime::evaluate_flow(&graph, "IncomeAmounts", movements).unwrap();
    assert_eq!(amounts, serde_json::json!([125000]));

    let movements = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-batch.json"
    ))
    .unwrap();
    let ordered =
        axl_compiler::next::runtime::evaluate_flow(&graph, "SortMovementsNewest", movements)
            .unwrap();
    assert_eq!(ordered[0]["id"], "movement-002");
    assert_eq!(ordered[1]["id"], "movement-001");

    let movements = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-batch.json"
    ))
    .unwrap();
    let grouped =
        axl_compiler::next::runtime::evaluate_flow(&graph, "GroupMovementsByCategory", movements)
            .unwrap();
    assert_eq!(grouped["consulting"][0]["id"], "movement-001");
    assert_eq!(grouped["software"][0]["id"], "movement-002");

    let categories = axl_compiler::next::runtime::evaluate_flow(
        &graph,
        "DefaultCategories",
        serde_json::Value::Null,
    )
    .unwrap();
    assert_eq!(categories, serde_json::json!(["consulting", "software"]));

    let response =
        axl_compiler::next::http::dispatch(&graph, "get", "/categories", serde_json::Value::Null);
    assert_eq!(response.status, 200);
    assert_eq!(response.body, serde_json::json!(["consulting", "software"]));

    let movements = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-batch.json"
    ))
    .unwrap();
    let views =
        axl_compiler::next::runtime::evaluate_flow(&graph, "BuildMovementViewsParallel", movements)
            .unwrap();
    assert_eq!(views[0]["id"], "movement-001");
    assert_eq!(views[1]["id"], "movement-002");

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

    let movement = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-valid.json"
    ))
    .unwrap();
    let mut event_runtime = axl_compiler::next::runtime::BuiltinRuntime::new().unwrap();
    let announced = axl_compiler::next::runtime::evaluate_flow_with_runtime(
        &graph,
        "SaveAndAnnounce",
        movement,
        &mut event_runtime,
    )
    .unwrap();
    assert_eq!(announced["ok"]["id"], "movement-001");
    let tags = axl_compiler::next::runtime::evaluate_flow_with_runtime(
        &graph,
        "ListMovementTags",
        serde_json::Value::Null,
        &mut event_runtime,
    )
    .unwrap();
    assert_eq!(tags["ok"], serde_json::json!(["persisted", "announced"]));

    let movement = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-valid.json"
    ))
    .unwrap();
    let mut job_runtime = axl_compiler::next::runtime::BuiltinRuntime::new().unwrap();
    let scheduled = axl_compiler::next::runtime::evaluate_flow_with_runtime(
        &graph,
        "ScheduleMovementPersist",
        movement,
        &mut job_runtime,
    )
    .unwrap();
    assert_eq!(scheduled["ok"]["id"], "movement-001");
    let executed = axl_compiler::next::runtime::run_due_jobs(&graph, &mut job_runtime).unwrap();
    assert!(executed >= 1);
    let found = axl_compiler::next::runtime::evaluate_flow_with_runtime(
        &graph,
        "FindMovement",
        serde_json::json!("movement-001"),
        &mut job_runtime,
    )
    .unwrap();
    assert_eq!(found["ok"]["id"], "movement-001");

    let invalid = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-invalid.json"
    ))
    .unwrap();
    let rejected =
        axl_compiler::next::runtime::evaluate_flow(&graph, "ValidateAndStoreMovement", invalid)
            .unwrap();
    assert_eq!(rejected["error"], "amount_must_be_positive");

    let movement = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-valid.json"
    ))
    .unwrap();
    let response = axl_compiler::next::http::dispatch(&graph, "post", "/movements", movement);
    assert_eq!(response.status, 200);
    assert_eq!(response.body["ok"]["id"], "movement-001");

    let batch = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-batch.json"
    ))
    .unwrap();
    let response = axl_compiler::next::http::dispatch(&graph, "post", "/balance", batch);
    assert_eq!(response.status, 200);
    assert_eq!(response.body, 80000);

    let mut runtime = axl_compiler::next::runtime::BuiltinRuntime::new().unwrap();
    let movement = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-valid.json"
    ))
    .unwrap();
    let stored = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/movements",
        movement,
    );
    assert_eq!(stored.status, 200);
    let found = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/movement-by-id",
        serde_json::json!("movement-001"),
    );
    assert_eq!(found.status, 200);
    assert_eq!(found.body["ok"]["id"], "movement-001");
    let found = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/movement-by-id/resilient",
        serde_json::json!("movement-001"),
    );
    assert_eq!(found.status, 200);
    assert_eq!(found.body["ok"]["id"], "movement-001");
    let found = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/movement-first",
        serde_json::json!({"ids": ["missing", "movement-001"]}),
    );
    assert_eq!(found.status, 200);
    assert_eq!(found.body["ok"]["id"], "movement-001");

    let mut headers = std::collections::BTreeMap::new();
    headers.insert("x-user".into(), "alice".into());
    headers.insert("cookie".into(), "sid=session-42".into());
    let me = axl_compiler::next::http::dispatch_with_headers(
        &graph,
        &mut runtime,
        "get",
        "/me",
        serde_json::Value::Null,
        &headers,
    );
    assert_eq!(me.status, 200);
    assert_eq!(me.body, "alice");
    let session = axl_compiler::next::http::dispatch_with_headers(
        &graph,
        &mut runtime,
        "get",
        "/session",
        serde_json::Value::Null,
        &headers,
    );
    assert_eq!(session.status, 200);
    assert_eq!(session.body, "session-42");
    let movement = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-valid.json"
    ))
    .unwrap();
    let preview = axl_compiler::next::http::dispatch_with_headers(
        &graph,
        &mut runtime,
        "post",
        "/client-preview",
        movement,
        &headers,
    );
    assert_eq!(preview.status, 200);
    assert_eq!(preview.body["ok"]["id"], "movement-001");

    let annotated = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/annotated/balance",
        batch,
    );
    assert_eq!(annotated.status, 200);
    assert_eq!(annotated.body, 80000);
    assert_eq!(
        annotated
            .headers
            .get("x-axl-middleware")
            .map(String::as_str),
        Some("ok")
    );
}
