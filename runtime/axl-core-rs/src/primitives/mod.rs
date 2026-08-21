pub mod io;
pub mod net;
pub mod text;
pub mod collections;
pub mod math;
pub mod system;
pub mod serialize;
pub mod crypto;

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
        "http_get", "http_post",
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
    ]
}
