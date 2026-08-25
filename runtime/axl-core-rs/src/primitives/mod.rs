pub mod io;
pub mod net;
pub mod text;
pub mod collections;
pub mod math;
pub mod system;
pub mod serialize;
pub mod crypto;
pub mod http;
pub mod db;
pub mod ws;
pub mod cache;
pub mod logging;
pub mod metrics;
pub mod validation;
pub mod uuid;
pub mod compress;
pub mod watch;
pub mod cron;
pub mod auth;
pub mod email;
pub mod ratelimit;
pub mod secret;
pub mod llm;
pub mod compiler;

use crate::ir::Value;

/// Errore di esecuzione primitiva
#[derive(Debug, Clone)]
pub struct PrimitiveError(pub String);

impl std::fmt::Display for PrimitiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PrimitiveError {}

/// Tipo di permission richiesto per una primitiva
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    Read,
    Write,
    Execute,
    Network,
    Crypto,
    System,
    Ai,
}

/// Metadata di una primitiva
pub struct PrimitiveInfo {
    pub name: &'static str,
    pub permission: Permission,
    pub description: &'static str,
}

/// Registry di tutte le primitive native
pub fn call_primitive(name: &str, args: &[Value]) -> Result<Value, PrimitiveError> {
    match name {
        // I/O — File
        "file_read" => io::file_read(args),
        "file_write" => io::file_write(args),
        "file_exists" => io::file_exists(args),
        "file_size" => io::file_size(args),
        "file_delete" => io::file_delete(args),
        "file_copy" => io::file_copy(args),
        "file_move" => io::file_move(args),
        "dir_create" => io::dir_create(args),
        "dir_list" => io::dir_list(args),
        "dir_delete" => io::dir_delete(args),

        // Network
        "http_get" => net::http_get(args),
        "http_post" => net::http_post(args),
        "http_put" => net::http_put(args),
        "http_delete" => net::http_delete(args),
        "http_patch" => net::http_patch(args),
        "http_download" => net::http_download(args),
        "http_server_create" => http::http_server_create(args).map_err(PrimitiveError),
        "http_server_route" => http::http_server_route(args).map_err(PrimitiveError),
        "http_server_static" => http::http_server_static(args).map_err(PrimitiveError),
        "http_server_listen" => http::http_server_listen(args).map_err(PrimitiveError),
        "http_response" => http::http_response(args).map_err(PrimitiveError),
        "http_response_json" => http::http_response_json(args).map_err(PrimitiveError),
        "http_response_html" => http::http_response_html(args).map_err(PrimitiveError),
        "http_response_error" => http::http_response_error(args).map_err(PrimitiveError),
        "http_request_method" => http::http_request_method(args).map_err(PrimitiveError),
        "http_request_path" => http::http_request_path(args).map_err(PrimitiveError),
        "http_request_query" => http::http_request_query(args).map_err(PrimitiveError),
        "http_request_body" => http::http_request_body(args).map_err(PrimitiveError),
        "http_request_header" => http::http_request_header(args).map_err(PrimitiveError),
        "http_server_state_get" => http::http_server_state_get(args).map_err(PrimitiveError),
        "http_server_state_set" => http::http_server_state_set(args).map_err(PrimitiveError),
        "axl_server_start" => http::axl_server_start(args).map_err(PrimitiveError),
        "http_server_api" => http::http_server_api(args).map_err(PrimitiveError),
        "axl_compile_frontend" => compiler::axl_compile_frontend(args),

        // Database
        "db_connect" => db::db_connect(args).map_err(PrimitiveError),
        "db_execute" => db::db_execute(args).map_err(PrimitiveError),
        "db_query" => db::db_query(args).map_err(PrimitiveError),
        "db_begin" => db::db_begin(args).map_err(PrimitiveError),
        "db_commit" => db::db_commit(args).map_err(PrimitiveError),
        "db_rollback" => db::db_rollback(args).map_err(PrimitiveError),
        "db_tables" => db::db_tables(args).map_err(PrimitiveError),
        "db_columns" => db::db_columns(args).map_err(PrimitiveError),
        "db_count" => db::db_count(args).map_err(PrimitiveError),
        "db_insert" => db::db_insert(args).map_err(PrimitiveError),
        "db_update" => db::db_update(args).map_err(PrimitiveError),
        "db_delete" => db::db_delete(args).map_err(PrimitiveError),

        // WebSocket
        "ws_server_create" => ws::ws_server_create(args).map_err(PrimitiveError),
        "ws_connect" => ws::ws_connect(args).map_err(PrimitiveError),
        "ws_send" => ws::ws_send(args).map_err(PrimitiveError),
        "ws_recv" => ws::ws_recv(args).map_err(PrimitiveError),
        "ws_broadcast" => ws::ws_broadcast(args).map_err(PrimitiveError),
        "ws_on_message" => ws::ws_on_message(args).map_err(PrimitiveError),

        // Cache
        "cache_create" => cache::cache_create(args).map_err(PrimitiveError),
        "cache_get" => cache::cache_get(args).map_err(PrimitiveError),
        "cache_set" => cache::cache_set(args).map_err(PrimitiveError),
        "cache_set_ttl" => cache::cache_set_ttl(args).map_err(PrimitiveError),
        "cache_delete" => cache::cache_delete(args).map_err(PrimitiveError),
        "cache_clear" => cache::cache_clear(args).map_err(PrimitiveError),
        "cache_size" => cache::cache_size(args).map_err(PrimitiveError),

        // Logging
        "log_info" => logging::log_info(args).map_err(PrimitiveError),
        "log_warn" => logging::log_warn(args).map_err(PrimitiveError),
        "log_error" => logging::log_error(args).map_err(PrimitiveError),
        "log_debug" => logging::log_debug(args).map_err(PrimitiveError),
        "log_set_level" => logging::log_set_level(args).map_err(PrimitiveError),
        "log_set_file" => logging::log_set_file(args).map_err(PrimitiveError),
        "log_json" => logging::log_json(args).map_err(PrimitiveError),

        // Metrics
        "metric_counter" => metrics::metric_counter(args).map_err(PrimitiveError),
        "metric_gauge" => metrics::metric_gauge(args).map_err(PrimitiveError),
        "metric_histogram" => metrics::metric_histogram(args).map_err(PrimitiveError),
        "metric_timer_start" => metrics::metric_timer_start(args).map_err(PrimitiveError),
        "metric_timer_stop" => metrics::metric_timer_stop(args).map_err(PrimitiveError),

        // Validation
        "validate_email" => validation::validate_email(args).map_err(PrimitiveError),
        "validate_url" => validation::validate_url(args).map_err(PrimitiveError),
        "validate_ip" => validation::validate_ip(args).map_err(PrimitiveError),
        "validate_uuid" => validation::validate_uuid(args).map_err(PrimitiveError),
        "validate_json" => validation::validate_json_str(args).map_err(PrimitiveError),
        "validate_regex" => validation::validate_regex(args).map_err(PrimitiveError),
        "validate_credit_card" => validation::validate_credit_card(args).map_err(PrimitiveError),
        "validate_phone" => validation::validate_phone(args).map_err(PrimitiveError),

        // UUID
        "uuid_v4" => uuid::uuid_v4(args).map_err(PrimitiveError),
        "uuid_v5" => uuid::uuid_v5(args).map_err(PrimitiveError),
        "uuid_parse" => uuid::uuid_parse(args).map_err(PrimitiveError),
        "uuid_validate" => uuid::uuid_validate(args).map_err(PrimitiveError),

        // Compression
        "gzip_compress" => compress::gzip_compress(args).map_err(PrimitiveError),
        "gzip_decompress" => compress::gzip_decompress(args).map_err(PrimitiveError),
        "zstd_compress" => compress::zstd_compress(args).map_err(PrimitiveError),
        "zstd_decompress" => compress::zstd_decompress(args).map_err(PrimitiveError),
        "brotli_compress" => compress::brotli_compress(args).map_err(PrimitiveError),
        "brotli_decompress" => compress::brotli_decompress(args).map_err(PrimitiveError),

        // File Watch
        "watch_create" => watch::watch_create(args).map_err(PrimitiveError),
        "watch_add" => watch::watch_add(args).map_err(PrimitiveError),
        "watch_remove" => watch::watch_remove(args).map_err(PrimitiveError),
        "watch_poll" => watch::watch_poll(args).map_err(PrimitiveError),
        "watch_close" => watch::watch_close(args).map_err(PrimitiveError),

        // Cron
        "cron_create" => cron::cron_create(args).map_err(PrimitiveError),
        "cron_add" => cron::cron_add(args).map_err(PrimitiveError),
        "cron_remove" => cron::cron_remove(args).map_err(PrimitiveError),
        "cron_start" => cron::cron_start(args).map_err(PrimitiveError),
        "cron_stop" => cron::cron_stop(args).map_err(PrimitiveError),
        "cron_list" => cron::cron_list(args).map_err(PrimitiveError),

        // Auth
        "auth_hash_password" => auth::auth_hash_password(args).map_err(PrimitiveError),
        "auth_verify_password" => auth::auth_verify_password(args).map_err(PrimitiveError),
        "auth_jwt_create" => auth::auth_jwt_create(args).map_err(PrimitiveError),
        "auth_jwt_verify" => auth::auth_jwt_verify(args).map_err(PrimitiveError),
        "auth_jwt_decode" => auth::auth_jwt_decode(args).map_err(PrimitiveError),
        "session_create" => auth::session_create(args).map_err(PrimitiveError),
        "session_get" => auth::session_get(args).map_err(PrimitiveError),
        "session_destroy" => auth::session_destroy(args).map_err(PrimitiveError),

        // Email
        "email_send" => email::email_send(args).map_err(PrimitiveError),
        "email_send_html" => email::email_send_html(args).map_err(PrimitiveError),
        "email_send_attach" => email::email_send_attach(args).map_err(PrimitiveError),
        "email_template" => email::email_template(args).map_err(PrimitiveError),

        // Rate Limit
        "ratelimit_create" => ratelimit::ratelimit_create(args).map_err(PrimitiveError),
        "ratelimit_check" => ratelimit::ratelimit_check(args).map_err(PrimitiveError),
        "ratelimit_reset" => ratelimit::ratelimit_reset(args).map_err(PrimitiveError),

        // Secrets
        "secret_store" => secret::secret_store(args).map_err(PrimitiveError),
        "secret_get" => secret::secret_get(args).map_err(PrimitiveError),
        "secret_delete" => secret::secret_delete(args).map_err(PrimitiveError),
        "secret_list" => secret::secret_list(args).map_err(PrimitiveError),

        // LLM
        "llm_generate" => llm::llm_generate(args).map_err(PrimitiveError),
        "llm_reason" => llm::llm_reason(args).map_err(PrimitiveError),
        "llm_classify" => llm::llm_classify(args).map_err(PrimitiveError),
        "llm_extract" => llm::llm_extract(args).map_err(PrimitiveError),
        "llm_embed" => llm::llm_embed(args).map_err(PrimitiveError),
        "llm_similarity" => llm::llm_similarity(args).map_err(PrimitiveError),

        // Text
        "text_upper" => text::text_upper(args),
        "text_lower" => text::text_lower(args),
        "text_trim" => text::text_trim(args),
        "text_replace" => text::text_replace(args),
        "text_split" => text::text_split(args),
        "text_join" => text::text_join(args),
        "text_find" => text::text_find(args),
        "text_contains" => text::text_contains(args),
        "text_matches" => text::text_matches(args),
        "text_length" => text::text_length(args),
        "text_reverse" => text::text_reverse(args),
        "text_lines" => text::text_lines(args),
        "text_extract" => text::text_extract(args),

        // Collections
        "list_new" => collections::list_new(args),
        "list_push" => collections::list_push(args),
        "list_length" => collections::list_length(args),
        "list_contains" => collections::list_contains(args),
        "list_sort" => collections::list_sort(args),
        "list_reverse" => collections::list_reverse(args),
        "list_unique" => collections::list_unique(args),
        "list_flatten" => collections::list_flatten(args),
        "list_slice" => collections::list_slice(args),
        "list_filter" => collections::list_filter(args),
        "list_map" => collections::list_map_op(args),
        "list_head" => collections::list_head(args),
        "list_tail" => collections::list_tail(args),
        "list_pop" => collections::list_pop(args),
        "list_index" => collections::list_index(args),
        "list_diff" => collections::list_diff(args),
        "list_sum" => collections::list_sum(args),
        "map_new" => collections::map_new(args),
        "map_get" => collections::map_get(args),
        "map_set" => collections::map_set(args),
        "map_keys" => collections::map_keys(args),
        "map_values" => collections::map_values(args),
        "map_contains" => collections::map_contains(args),
        "map_delete" => collections::map_delete(args),
        "map_merge" => collections::map_merge(args),
        "map_entries" => collections::map_entries(args),

        // Math
        "math_add" => math::math_add(args),
        "math_sub" => math::math_sub(args),
        "math_mul" => math::math_mul(args),
        "math_div" => math::math_div(args),
        "math_mod" => math::math_mod(args),
        "math_pow" => math::math_pow(args),
        "math_abs" => math::math_abs(args),
        "math_min" => math::math_min(args),
        "math_max" => math::math_max(args),
        "math_clamp" => math::math_clamp(args),
        "math_random" => math::math_random(args),
        "math_random_range" => math::math_random_range(args),
        "math_sum" => math::math_sum_list(args),
        "math_average" => math::math_average(args),

        // System
        "env_get" => system::env_get(args),
        "env_set" => system::env_set(args),
        "env_list" => system::env_list(args),
        "time_now" => system::time_now(args),
        "time_format" => system::time_format(args),
        "time_sleep" => system::time_sleep(args),
        "path_join" => system::path_join(args),
        "path_absolute" => system::path_absolute(args),
        "path_parent" => system::path_parent(args),
        "path_filename" => system::path_filename(args),
        "path_extension" => system::path_extension(args),
        "path_exists" => system::path_exists(args),
        "temp_dir" => system::temp_dir(args),
        "temp_file" => system::temp_file(args),
        "sys_hostname" => system::sys_hostname(args),
        "sys_os" => system::sys_os(args),
        "sys_arch" => system::sys_arch(args),
        "process_run" => system::process_run(args),
        "process_output" => system::process_output(args),

        // Serialize
        "json_parse" => serialize::json_parse(args),
        "json_stringify" => serialize::json_stringify(args),
        "json_validate" => serialize::json_validate(args),

        // Crypto
        "hash_sha256" => crypto::hash_sha256(args),
        "hash_blake3" => crypto::hash_blake3(args),
        "hash_md5" => crypto::hash_md5(args),
        "encode_base64" => crypto::encode_base64(args),
        "decode_base64" => crypto::decode_base64(args),
        "encode_hex" => crypto::encode_hex(args),
        "decode_hex" => crypto::decode_hex(args),
        "crypto_random_bytes" => crypto::crypto_random_bytes(args),

        _ => Err(PrimitiveError(format!("unknown primitive '{name}'"))),
    }
}

/// Lista di tutte le primitive disponibili
pub fn available_primitives() -> Vec<&'static str> {
    vec![
        // I/O
        "file_read", "file_write", "file_exists", "file_size", "file_delete",
        "file_copy", "file_move", "dir_create", "dir_list", "dir_delete",
        // Network
        "http_get", "http_post", "http_put", "http_delete", "http_patch",
        "http_download", "http_server_create", "http_server_route",
        "http_server_static", "http_server_listen", "http_response",
        "http_response_json", "http_response_html", "http_response_error",
        "http_request_method", "http_request_path", "http_request_query",
        "http_request_body", "http_request_header",
        "http_server_state_get", "http_server_state_set",
        // Database
        "db_connect", "db_execute", "db_query", "db_begin", "db_commit",
        "db_rollback", "db_tables", "db_columns", "db_count",
        "db_insert", "db_update", "db_delete",
        // WebSocket
        "ws_server_create", "ws_connect", "ws_send", "ws_recv",
        "ws_broadcast", "ws_on_message",
        // Cache
        "cache_create", "cache_get", "cache_set", "cache_set_ttl",
        "cache_delete", "cache_clear", "cache_size",
        // Logging
        "log_info", "log_warn", "log_error", "log_debug",
        "log_set_level", "log_set_file", "log_json",
        // Metrics
        "metric_counter", "metric_gauge", "metric_histogram",
        "metric_timer_start", "metric_timer_stop",
        // Validation
        "validate_email", "validate_url", "validate_ip", "validate_uuid",
        "validate_json", "validate_regex", "validate_credit_card", "validate_phone",
        // UUID
        "uuid_v4", "uuid_v5", "uuid_parse", "uuid_validate",
        // Compression
        "gzip_compress", "gzip_decompress", "zstd_compress", "zstd_decompress",
        "brotli_compress", "brotli_decompress",
        // File Watch
        "watch_create", "watch_add", "watch_remove", "watch_poll", "watch_close",
        // Cron
        "cron_create", "cron_add", "cron_remove", "cron_start", "cron_stop", "cron_list",
        // Auth
        "auth_hash_password", "auth_verify_password", "auth_jwt_create",
        "auth_jwt_verify", "auth_jwt_decode", "session_create", "session_get",
        "session_destroy",
        // Email
        "email_send", "email_send_html", "email_send_attach", "email_template",
        // Rate Limit
        "ratelimit_create", "ratelimit_check", "ratelimit_reset",
        // Secrets
        "secret_store", "secret_get", "secret_delete", "secret_list",
        // Text
        "text_upper", "text_lower", "text_trim", "text_replace", "text_split",
        "text_join", "text_find", "text_contains", "text_matches", "text_length",
        "text_reverse", "text_lines", "text_extract",
        // Collections
        "list_new", "list_push", "list_length", "list_contains", "list_sort",
        "list_reverse", "list_unique", "list_flatten", "list_slice", "list_filter",
        "list_map", "list_head", "list_tail", "list_pop", "list_index",
        "list_diff", "list_sum",
        "map_new", "map_get", "map_set", "map_keys", "map_values",
        "map_contains", "map_delete", "map_merge", "map_entries",
        // Math
        "math_add", "math_sub", "math_mul", "math_div", "math_mod",
        "math_pow", "math_abs", "math_min", "math_max", "math_clamp",
        "math_random", "math_random_range", "math_sum", "math_average",
        // System
        "env_get", "env_set", "env_list", "time_now", "time_format",
        "time_sleep", "path_join", "path_absolute", "path_parent", "path_filename",
        "path_extension", "path_exists", "temp_dir", "temp_file",
        "sys_hostname", "sys_os", "sys_arch", "process_run", "process_output",
        // Serialize
        "json_parse", "json_stringify", "json_validate",
        // Crypto
        "hash_sha256", "hash_blake3", "hash_md5",
        "encode_base64", "decode_base64", "encode_hex", "decode_hex",
        "crypto_random_bytes",
        // LLM
        "llm_generate", "llm_reason", "llm_classify", "llm_extract",
        "llm_embed", "llm_similarity",
        // Server
        "axl_server_start", "http_server_api",
        "axl_compile_frontend",
    ]
}
