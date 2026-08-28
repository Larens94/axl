use axl_compiler::compile_source;

const EXAMPLES: [(&str, &str); 10] = [
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
    (
        "balance-ui",
        include_str!("../../../examples/apps/balance-ui.axl"),
    ),
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
            "AXL-D901",
            include_str!("../../../examples/invalid/flow-transactions.axl"),
        ),
        (
            "AXL-D902",
            include_str!("../../../examples/invalid/flow-migrations.axl"),
        ),
        (
            "AXL-D903",
            include_str!("../../../examples/invalid/flow-queries.axl"),
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
        (
            "AXL-P951",
            include_str!("../../../examples/invalid/ui-syntax.axl"),
        ),
        (
            "AXL-U904",
            include_str!("../../../examples/invalid/ui-unknown-flow.axl"),
        ),
        (
            "AXL-U905",
            include_str!("../../../examples/invalid/ui-flow-mismatch.axl"),
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

    let batch: serde_json::Value = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-batch.json"
    ))
    .unwrap();
    let response = axl_compiler::next::http::dispatch(&graph, "post", "/balance", batch.clone());
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

    let cache_entry = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/balance-cache.json"
    ))
    .unwrap();
    let cached = axl_compiler::next::runtime::evaluate_flow_with_runtime(
        &graph,
        "CacheBalanceSnapshot",
        cache_entry,
        &mut runtime,
    )
    .unwrap();
    assert_eq!(cached, serde_json::json!({"ok": "80000"}));
    let loaded = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/cache/balance/get",
        serde_json::json!("ledger:demo"),
    );
    assert_eq!(loaded.status, 200);
    assert_eq!(loaded.body, serde_json::json!({"ok": "80000"}));
    let invalidated = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/cache/balance/invalidate",
        serde_json::json!("ledger:demo"),
    );
    assert_eq!(invalidated.status, 200);
    assert_eq!(invalidated.body, serde_json::json!({"ok": true}));
    let miss = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/cache/balance/get",
        serde_json::json!("ledger:demo"),
    );
    assert_eq!(miss.status, 422);
    assert_eq!(miss.body, serde_json::json!({"error": "cache_miss"}));

    let logged = axl_compiler::next::runtime::evaluate_flow_with_runtime(
        &graph,
        "RecordTwoObservabilityLines",
        serde_json::Value::Null,
        &mut runtime,
    )
    .unwrap();
    assert_eq!(
        logged,
        serde_json::json!({"ok": ["ledger.balance", "ledger.balance"]})
    );
    let metric = axl_compiler::next::runtime::evaluate_flow_with_runtime(
        &graph,
        "ObserveMetricTwice",
        serde_json::Value::Null,
        &mut runtime,
    )
    .unwrap();
    assert_eq!(metric, serde_json::json!({"ok": 2}));
    let spans = axl_compiler::next::runtime::evaluate_flow_with_runtime(
        &graph,
        "TraceObservabilitySpan",
        serde_json::Value::Null,
        &mut runtime,
    )
    .unwrap();
    assert_eq!(spans, serde_json::json!({"ok": ["CalculateLedgerBalance"]}));
    let first_log = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/observability/log",
        serde_json::json!("http.write"),
    );
    assert_eq!(first_log.status, 200);
    assert_eq!(first_log.body, serde_json::json!({"ok": null}));
    let second_log = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/observability/log",
        serde_json::json!("http.write"),
    );
    assert_eq!(second_log.status, 200);
    let listed = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/observability/logs",
        serde_json::Value::Null,
    );
    assert_eq!(listed.status, 200);
    assert_eq!(
        listed.body,
        serde_json::json!({
            "ok": [
                "ledger.balance",
                "ledger.balance",
                "http.write",
                "http.write"
            ]
        })
    );

    let annotated = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/annotated/balance",
        batch.clone(),
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

    for _ in 0..5 {
        let allowed = axl_compiler::next::http::dispatch_with_runtime(
            &graph,
            &mut runtime,
            "post",
            "/limited/balance",
            batch.clone(),
        );
        assert_eq!(allowed.status, 200);
        assert_eq!(allowed.body, 80000);
    }
    let limited = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/limited/balance",
        batch.clone(),
    );
    assert_eq!(limited.status, 429);
    assert_eq!(
        limited.body,
        serde_json::json!({ "error": "rate_limit_exceeded" })
    );

    let cors = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "post",
        "/cors/balance",
        batch.clone(),
    );
    assert_eq!(cors.status, 200);
    assert_eq!(cors.body, 80000);
    assert_eq!(
        cors.headers
            .get("access-control-allow-origin")
            .map(String::as_str),
        Some("*")
    );
    assert_eq!(
        cors.headers
            .get("access-control-allow-methods")
            .map(String::as_str),
        Some("GET,POST,OPTIONS")
    );
    let preflight = axl_compiler::next::http::dispatch_with_runtime(
        &graph,
        &mut runtime,
        "options",
        "/cors/balance",
        serde_json::Value::Null,
    );
    assert_eq!(preflight.status, 204);
    assert_eq!(
        preflight
            .headers
            .get("access-control-allow-origin")
            .map(String::as_str),
        Some("*")
    );
    assert_eq!(
        preflight
            .headers
            .get("access-control-allow-methods")
            .map(String::as_str),
        Some("GET,POST,OPTIONS")
    );

    let jwt_missing = axl_compiler::next::http::dispatch_with_authorization(
        &graph,
        &mut runtime,
        "post",
        "/jwt/balance",
        batch.clone(),
        None,
    );
    assert_eq!(jwt_missing.status, 401);
    let jwt_denied = axl_compiler::next::http::dispatch_with_authorization(
        &graph,
        &mut runtime,
        "post",
        "/jwt/balance",
        batch.clone(),
        Some("Bearer not-a-jwt"),
    );
    assert_eq!(jwt_denied.status, 403);
    let token = axl_compiler::next::runtime::encode_hs256_jwt(
        "axl-cashflow-demo-jwt",
        &serde_json::json!({"sub": "alice", "iss": "axl-cashflow"}),
    )
    .expect("demo jwt");
    let jwt_ok = axl_compiler::next::http::dispatch_with_authorization(
        &graph,
        &mut runtime,
        "post",
        "/jwt/balance",
        batch,
        Some(&format!("Bearer {token}")),
    );
    assert_eq!(jwt_ok.status, 200);
    assert_eq!(jwt_ok.body, 80000);

    let commit_pair = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-pair-commit.json"
    ))
    .unwrap();
    {
        let mut tx_runtime = axl_compiler::next::runtime::BuiltinRuntime::new().unwrap();
        let committed = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "CommitTwoDurableMovements",
            commit_pair,
            &mut tx_runtime,
        )
        .unwrap();
        assert_eq!(committed["ok"]["id"], "movement-tx-c02");
    }
    {
        let mut fresh = axl_compiler::next::runtime::BuiltinRuntime::new().unwrap();
        let first = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "FindDurableMovement",
            serde_json::json!("movement-tx-c01"),
            &mut fresh,
        )
        .unwrap();
        assert_eq!(first["ok"]["id"], "movement-tx-c01");
        let second = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "FindDurableMovement",
            serde_json::json!("movement-tx-c02"),
            &mut fresh,
        )
        .unwrap();
        assert_eq!(second["ok"]["id"], "movement-tx-c02");
    }

    let rollback_pair = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-pair-rollback.json"
    ))
    .unwrap();
    {
        let mut tx_runtime = axl_compiler::next::runtime::BuiltinRuntime::new().unwrap();
        let rolled = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "RollbackTwoDurableMovements",
            rollback_pair,
            &mut tx_runtime,
        )
        .unwrap();
        assert_eq!(rolled, serde_json::json!({"ok": null}));
    }
    {
        let mut fresh = axl_compiler::next::runtime::BuiltinRuntime::new().unwrap();
        let missing = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "FindDurableMovement",
            serde_json::json!("movement-tx-r01"),
            &mut fresh,
        )
        .unwrap();
        assert_eq!(missing["error"], "not_found");
        let missing = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "FindDurableMovement",
            serde_json::json!("movement-tx-r02"),
            &mut fresh,
        )
        .unwrap();
        assert_eq!(missing["error"], "not_found");
    }

    let migration_v1 = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/migration-v1.json"
    ))
    .unwrap();
    let migration_v2 = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/migration-v2.json"
    ))
    .unwrap();
    {
        let mut mig_runtime = axl_compiler::next::runtime::BuiltinRuntime::new().unwrap();
        loop {
            let status = axl_compiler::next::runtime::evaluate_flow_with_runtime(
                &graph,
                "DurableMigrationStatus",
                serde_json::Value::Null,
                &mut mig_runtime,
            )
            .unwrap();
            let head = status["ok"].as_str().unwrap().to_string();
            if head == "0" {
                break;
            }
            let rolled = axl_compiler::next::runtime::evaluate_flow_with_runtime(
                &graph,
                "RollbackDurableMigration",
                serde_json::json!(head),
                &mut mig_runtime,
            )
            .unwrap();
            assert_eq!(rolled["ok"], head);
        }
        let applied = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "ApplyDurableMigration",
            migration_v1,
            &mut mig_runtime,
        )
        .unwrap();
        assert_eq!(applied, serde_json::json!({"ok": "v1"}));
        let applied = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "ApplyDurableMigration",
            migration_v2,
            &mut mig_runtime,
        )
        .unwrap();
        assert_eq!(applied, serde_json::json!({"ok": "v2"}));
        let status = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "DurableMigrationStatus",
            serde_json::Value::Null,
            &mut mig_runtime,
        )
        .unwrap();
        assert_eq!(status, serde_json::json!({"ok": "v2"}));
    }
    {
        let mut fresh = axl_compiler::next::runtime::BuiltinRuntime::new().unwrap();
        let status = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "DurableMigrationStatus",
            serde_json::Value::Null,
            &mut fresh,
        )
        .unwrap();
        assert_eq!(status, serde_json::json!({"ok": "v2"}));
        let rolled = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "RollbackDurableMigration",
            serde_json::json!("v2"),
            &mut fresh,
        )
        .unwrap();
        assert_eq!(rolled, serde_json::json!({"ok": "v2"}));
        let status = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "DurableMigrationStatus",
            serde_json::Value::Null,
            &mut fresh,
        )
        .unwrap();
        assert_eq!(status, serde_json::json!({"ok": "v1"}));
    }
    {
        let mut fresh = axl_compiler::next::runtime::BuiltinRuntime::new().unwrap();
        let status = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "DurableMigrationStatus",
            serde_json::Value::Null,
            &mut fresh,
        )
        .unwrap();
        assert_eq!(status, serde_json::json!({"ok": "v1"}));
        let rolled = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "RollbackDurableMigration",
            serde_json::json!("v1"),
            &mut fresh,
        )
        .unwrap();
        assert_eq!(rolled, serde_json::json!({"ok": "v1"}));
        let status = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "DurableMigrationStatus",
            serde_json::Value::Null,
            &mut fresh,
        )
        .unwrap();
        assert_eq!(status, serde_json::json!({"ok": "0"}));
    }

    let query_movements: [serde_json::Value; 3] = [
        serde_json::from_str(include_str!(
            "../../../examples/apps/inputs/movement-query-q01.json"
        ))
        .unwrap(),
        serde_json::from_str(include_str!(
            "../../../examples/apps/inputs/movement-query-q02.json"
        ))
        .unwrap(),
        serde_json::from_str(include_str!(
            "../../../examples/apps/inputs/movement-query-q03.json"
        ))
        .unwrap(),
    ];
    let query_spec: serde_json::Value = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/movement-query-spec.json"
    ))
    .unwrap();
    {
        let mut query_runtime = axl_compiler::next::runtime::BuiltinRuntime::new().unwrap();
        for movement in &query_movements {
            let saved = axl_compiler::next::runtime::evaluate_flow_with_runtime(
                &graph,
                "SaveDurableMovement",
                movement.clone(),
                &mut query_runtime,
            )
            .unwrap();
            assert_eq!(saved["ok"]["id"], movement["id"]);
        }
        let page = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "QueryDurableMovements",
            query_spec.clone(),
            &mut query_runtime,
        )
        .unwrap();
        assert_eq!(page["ok"]["total"], 2);
        assert_eq!(page["ok"]["limit"], 1);
        assert_eq!(page["ok"]["offset"], 0);
        assert_eq!(page["ok"]["items"].as_array().unwrap().len(), 1);
        assert_eq!(page["ok"]["items"][0]["id"], "movement-q03");
    }
    {
        let mut fresh = axl_compiler::next::runtime::BuiltinRuntime::new().unwrap();
        let page = axl_compiler::next::runtime::evaluate_flow_with_runtime(
            &graph,
            "QueryDurableMovements",
            query_spec,
            &mut fresh,
        )
        .unwrap();
        assert_eq!(page["ok"]["total"], 2);
        assert_eq!(page["ok"]["items"][0]["id"], "movement-q03");
    }
}

#[test]
fn import_demo_compiles_from_file_and_round_trips() {
    use std::path::Path;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/apps/import-demo.axl");
    let compiled = axl_compiler::compile_file(&path)
        .unwrap()
        .unwrap_or_else(|diagnostics| panic!("import-demo failed: {diagnostics:#?}"));

    assert_eq!(compiled.graph.schema, "ax-ir/4.0");
    assert_eq!(compiled.graph.app, "ImportDemo");
    let decoded = axl_compiler::next::packed::decode(&compiled.matrix)
        .unwrap_or_else(|error| panic!("import-demo packed IR failed: {error}"));
    assert_eq!(decoded, compiled.graph);

    let formatted = axl_compiler::compile_source_at(&compiled.source, Some(&path)).unwrap_or_else(
        |diagnostics| panic!("import-demo formatted source failed: {diagnostics:#?}"),
    );
    assert_eq!(formatted.graph, compiled.graph);

    let balance = serde_json::json!({
        "income": 125000,
        "expense": 45000
    });
    let calculated = axl_compiler::next::runtime::evaluate_flow(
        &compiled.graph,
        "CalculateBalance",
        balance.clone(),
    )
    .unwrap();
    assert_eq!(calculated, 80000);

    let demo = axl_compiler::next::runtime::evaluate_flow(&compiled.graph, "DemoBalance", balance)
        .unwrap();
    assert_eq!(demo, 80000);
}

#[test]
fn import_invalid_examples_report_stable_codes() {
    use std::path::Path;

    let cases = [
        (
            "AXL-P931",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/invalid/import-missing.axl"),
        ),
        (
            "AXL-N002",
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/invalid/import-duplicate.axl"),
        ),
    ];

    for (code, path) in cases {
        let diagnostics = axl_compiler::compile_file(&path)
            .unwrap()
            .expect_err("example must be rejected");
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.code == code),
            "missing {code} for {}: {diagnostics:#?}",
            path.display()
        );
    }
}

#[test]
fn balance_ui_manifest_and_render_are_executable() {
    let compiled = compile_source(include_str!("../../../examples/apps/balance-ui.axl"))
        .unwrap_or_else(|diagnostics| panic!("balance-ui failed: {diagnostics:#?}"));
    let manifest = axl_compiler::next::ui::ui_manifest(&compiled.graph);
    assert_eq!(manifest["protocol"], "axl-ui/1");
    assert_eq!(manifest["uis"][0]["pages"][0]["flow"], "CalculateBalance");

    let balance =
        serde_json::from_str(include_str!("../../../examples/apps/inputs/balance.json")).unwrap();
    let rendered =
        axl_compiler::next::ui::render_page(&compiled.graph, "/balance", balance).unwrap();
    assert_eq!(rendered.data, serde_json::json!(80000));
    assert!(rendered.html.contains("80000"));
    assert!(rendered.html.contains("axl-ui/1"));
}

#[test]
fn check_json_success_envelope_is_stable() {
    use axl_compiler::next::diagnostic::CheckReport;
    use std::path::Path;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/apps/cashflow-core.axl");
    let compiled = axl_compiler::compile_file(&path)
        .unwrap()
        .unwrap_or_else(|diagnostics| panic!("cashflow-core failed: {diagnostics:#?}"));
    let report = CheckReport::success(
        Some(&path),
        &compiled.graph.app,
        &compiled.graph.schema,
        compiled.graph.nodes.len(),
        compiled.graph.edges.len(),
    );
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["protocol"], "axl-check/1");
    assert_eq!(json["ok"], true);
    assert!(
        json["path"]
            .as_str()
            .unwrap()
            .ends_with("cashflow-core.axl")
    );
    assert_eq!(json["app"], "CashflowCore");
    assert_eq!(json["schema"], "ax-ir/4.0");
    assert!(json["nodes"].as_u64().unwrap() > 0);
    assert!(json["edges"].as_u64().unwrap() > 0);
    assert!(
        json.get("diagnostics").is_none() || json["diagnostics"].as_array().unwrap().is_empty()
    );
}

#[test]
fn check_json_failure_envelope_reports_stable_codes() {
    use axl_compiler::next::diagnostic::CheckReport;
    use std::path::Path;

    let cases = [
        (
            "AXL-X817",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/invalid/flow-calls.axl"),
        ),
        (
            "AXL-P951",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/invalid/ui-syntax.axl"),
        ),
        (
            "AXL-P931",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/invalid/import-missing.axl"),
        ),
    ];

    for (code, path) in cases {
        let diagnostics = axl_compiler::compile_file(&path)
            .unwrap()
            .expect_err("example must be rejected");
        let report = CheckReport::failure(Some(&path), diagnostics);
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["protocol"], "axl-check/1");
        assert_eq!(json["ok"], false);
        assert!(
            json["path"]
                .as_str()
                .unwrap()
                .ends_with(path.file_name().unwrap().to_str().expect("path file name"))
        );
        let items = json["diagnostics"].as_array().expect("diagnostics array");
        assert!(
            items.iter().any(|item| item["code"] == code),
            "missing {code} in {items:#?}"
        );
        let matched = items
            .iter()
            .find(|item| item["code"] == code)
            .expect("matched diagnostic");
        assert!(!matched["message"].as_str().unwrap().is_empty());
        assert!(matched["span"]["line"].as_u64().unwrap() >= 1);
        assert_eq!(matched["severity"], "error");
        if path.ends_with("import-missing.axl") {
            assert_eq!(matched["phase"], "imports");
        }
    }
}

#[test]
fn agent_authored_ledger_compiles_and_renders_saldo() {
    use axl_compiler::next::ui;
    use std::path::Path;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/apps/ledger.axl");
    let compilation = axl_compiler::compile_file(&path).unwrap().unwrap();
    assert_eq!(compilation.graph.app, "LibroCassa");
    let saldo_input: serde_json::Value = serde_json::from_str(include_str!(
        "../../../examples/apps/inputs/ledger-saldo.json"
    ))
    .unwrap();
    let rendered = ui::render_page(&compilation.graph, "/saldo", saldo_input).unwrap();
    assert_eq!(rendered.data, serde_json::json!(108000));
    assert!(rendered.html.contains("108000"));
}
