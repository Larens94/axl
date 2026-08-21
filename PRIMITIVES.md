# AXL 3.0 — Complete Primitive Taxonomy

Every Rust capability mapped to an agent-native primitive.
Agents combine these primitives to build any software.

## Design Principles

1. **Uniform interface**: `primitive_name(args...) -> result`
2. **Type-safe**: Every primitive has clear input/output types
3. **Composable**: Primitives return values that other primitives consume
4. **Safe**: Wrapped in AXL's permission/policy system
5. **Agent-native**: Optimized for LLM generation and reasoning

---

## 1. I/O — File System

```axl
// Read/Write
file_read(path: string) -> bytes
file_write(path: string, data: bytes) -> bool
file_append(path: string, data: bytes) -> bool
file_delete(path: string) -> bool
file_exists(path: string) -> bool
file_size(path: string) -> int
file_copy(src: string, dst: string) -> bool
file_move(src: string, dst: string) -> bool

// Directory
dir_create(path: string) -> bool
dir_delete(path: string) -> bool
dir_list(path: string) -> list<string>
dir_walk(path: string) -> list<file_info>

// Metadata
file_metadata(path: string) -> map
file_watch(path: string) -> stream<file_event>

// Memory-mapped
mmap_read(path: string) -> bytes
mmap_write(path: string, data: bytes) -> bool
```

## 2. I/O — Network

```axl
// HTTP
http_get(url: string, headers: map) -> response
http_post(url: string, body: bytes, headers: map) -> response
http_put(url: string, body: bytes, headers: map) -> response
http_delete(url: string, headers: map) -> response
http_request(method: string, url: string, options: map) -> response

// TCP/UDP
tcp_connect(host: string, port: int) -> connection
tcp_listen(host: string, port: int) -> listener
tcp_send(conn: connection, data: bytes) -> bool
tcp_recv(conn: connection, n: int) -> bytes
udp_send(host: string, port: int, data: bytes) -> bool
udp_listen(host: string, port: int) -> socket

// WebSocket
ws_connect(url: string) -> ws_connection
ws_send(conn: ws_connection, data: bytes) -> bool
ws_recv(conn: ws_connection) -> bytes
ws_close(conn: ws_connection) -> bool

// DNS
dns_resolve(host: string) -> list<string>
dns_lookup(host: string) -> map
```

## 3. I/O — Process

```axl
// Spawn
process_run(cmd: string, args: list<string>, env: map) -> process
process_output(cmd: string, args: list<string>) -> string
process_status(pid: int) -> int
process_kill(pid: int) -> bool
process_wait(pid: int) -> int

// Shell
shell_exec(cmd: string) -> string
shell_popen(cmd: string) -> stream<string>

// Pipe
pipe_create() -> pipe
pipe_read(pipe: pipe) -> bytes
pipe_write(pipe: pipe, data: bytes) -> bool
pipe_close(pipe: pipe) -> bool
```

## 4. I/O — Clipboard and System

```axl
// Clipboard
clipboard_read() -> string
clipboard_write(text: string) -> bool

// Notification
notify(title: string, body: string) -> bool

// Dialog
dialog_open(title: string, filter: string) -> string
dialog_save(title: string) -> string
dialog_message(title: string, body: string) -> bool

// System info
sys_info() -> map
sys_hostname() -> string
sys_username() -> string
sys_arch() -> string
sys_os() -> string
```

## 5. Data — Serialization

```axl
// JSON
json_parse(text: string) -> any
json_stringify(value: any) -> string
json_validate(text: string, schema: map) -> bool
json_merge(a: map, b: map) -> map
json_patch(doc: map, patch: map) -> map
json_pointer(doc: map, path: string) -> any

// TOML
toml_parse(text: string) -> map
toml_stringify(value: map) -> string

// YAML
yaml_parse(text: string) -> any
yaml_stringify(value: any) -> string

// CSV
csv_parse(text: string) -> list<map>
csv_stringify(rows: list<map>) -> string

// XML
xml_parse(text: string) -> map
xml_stringify(value: map) -> string
xml_xpath(doc: map, query: string) -> list

// Binary
bin_encode(value: any, format: string) -> bytes
bin_decode(data: bytes, format: string) -> any
```

## 6. Data — Text Processing

```axl
// Transform
text_upper(text: string) -> string
text_lower(text: string) -> string
text_trim(text: string) -> string
text_replace(text: string, from: string, to: string) -> string
text_reverse(text: string) -> string
text_repeat(text: string, n: int) -> string

// Split/Join
text_split(text: string, delimiter: string) -> list<string>
text_join(parts: list<string>, delimiter: string) -> string
text_lines(text: string) -> list<string>

// Search
text_find(text: string, pattern: string) -> list<int>
text_contains(text: string, pattern: string) -> bool
text_starts_with(text: string, prefix: string) -> bool
text_ends_with(text: string, suffix: string) -> bool
text_matches(text: string, regex: string) -> bool

// Extract
text_extract(text: string, pattern: string) -> list<string>
text_capture(text: string, pattern: string) -> map
text_between(text: string, start: string, end: string) -> string

// Format
text_format(template: string, vars: map) -> string
text_pad_left(text: string, width: int, char: string) -> string
text_pad_right(text: string, width: int, char: string) -> string
text_wrap(text: string, width: int) -> string
```

## 7. Data — Collection Operations

```axl
// List
list_new() -> list
list_push(list: list, item: any) -> list
list_pop(list: list) -> any
list_head(list: list) -> any
list_tail(list: list) -> list
list_length(list: list) -> int
list_contains(list: list, item: any) -> bool
list_index(list: list, item: any) -> int
list_slice(list: list, start: int, end: int) -> list
list_sort(list: list) -> list
list_reverse(list: list) -> list
list_unique(list: list) -> list
list_flatten(list: list) -> list
list_group_by(list: list, key: string) -> map
list_filter(list: list, predicate: string) -> list
list_map(list: list, transform: string) -> list
list_reduce(list: list, reducer: string, initial: any) -> any
list_zip(a: list, b: list) -> list
list_diff(a: list, b: list) -> list
list_intersect(a: list, b: list) -> list
list_union(a: list, b: list) -> list

// Map
map_new() -> map
map_get(map: map, key: string) -> any
map_set(map: map, key: string, value: any) -> map
map_delete(map: map, key: string) -> map
map_keys(map: map) -> list<string>
map_values(map: map) -> list
map_entries(map: map) -> list<tuple>
map_contains(map: map, key: string) -> bool
map_merge(a: map, b: map) -> map
map_filter(map: map, predicate: string) -> map
map_map(map: map, transform: string) -> map

// Set
set_new() -> set
set_add(set: set, item: any) -> set
set_remove(set: set, item: any) -> set
set_contains(set: set, item: any) -> bool
set_union(a: set, b: set) -> set
set_intersect(a: set, b: set) -> set
set_diff(a: set, b: set) -> set
```

## 8. Data — Math

```axl
// Basic
math_add(a: int, b: int) -> int
math_sub(a: int, b: int) -> int
math_mul(a: int, b: int) -> int
math_div(a: int, b: int) -> int
math_mod(a: int, b: int) -> int
math_pow(base: int, exp: int) -> int
math_sqrt(n: int) -> int
math_abs(n: int) -> int
math_neg(n: int) -> int

// Comparison
math_min(a: int, b: int) -> int
math_max(a: int, b: int) -> int
math_clamp(value: int, min: int, max: int) -> int
math_lerp(a: int, b: int, t: int) -> int

// Random
math_random() -> int
math_random_range(min: int, max: int) -> int
math_random_choice(list: list) -> any

// Aggregate
math_sum(list: list<int>) -> int
math_product(list: list<int>) -> int
math_average(list: list<int>) -> int
math_median(list: list<int>) -> int
math_stdev(list: list<int>) -> int
```

## 9. Data — Crypto

```axl
// Hash
hash_sha256(data: bytes) -> bytes
hash_sha512(data: bytes) -> bytes
hash_blake3(data: bytes) -> bytes
hash_md5(data: bytes) -> bytes

// HMAC
hmac_sha256(key: bytes, data: bytes) -> bytes

// Symmetric encryption
encrypt_aes_gcm(key: bytes, data: bytes, nonce: bytes) -> bytes
decrypt_aes_gcm(key: bytes, data: bytes, nonce: bytes) -> bytes
encrypt_chacha20(key: bytes, data: bytes, nonce: bytes) -> bytes
decrypt_chacha20(key: bytes, data: bytes, nonce: bytes) -> bytes

// Key derivation
derive_key(password: string, salt: bytes) -> bytes
derive_key_argon2(password: string, salt: bytes) -> bytes

// Asymmetric
keygen_ed25519() -> keypair
sign_ed25519(keypair: keypair, data: bytes) -> bytes
verify_ed25519(public_key: bytes, data: bytes, signature: bytes) -> bool
keygen_x25519() -> keypair
diffie_hellman(private_key: bytes, public_key: bytes) -> bytes

// Random
crypto_random_bytes(n: int) -> bytes
crypto_random_int() -> int
```

## 10. Data — Encoding

```axl
// Base encoding
encode_base64(data: bytes) -> string
decode_base64(text: string) -> bytes
encode_base32(data: bytes) -> string
decode_base32(text: string) -> bytes
encode_hex(data: bytes) -> string
decode_hex(text: string) -> bytes

// URL encoding
encode_url(text: string) -> string
decode_url(text: string) -> string

// Unicode
encode_utf8(text: string) -> bytes
decode_utf8(data: bytes) -> string
unicode_escape(text: string) -> string
unicode_unescape(text: string) -> string
```

## 11. Concurrency — Threading

```axl
// Thread
thread_spawn(closure: closure) -> thread
thread_join(thread: thread) -> any
thread_id() -> int
thread_sleep(ms: int) -> bool

// Channel
channel_create() -> channel
channel_send(channel: channel, value: any) -> bool
channel_recv(channel: channel) -> any
channel_close(channel: channel) -> bool

// Shared state
mutex_create() -> mutex
mutex_lock(mutex: mutex) -> bool
mutex_unlock(mutex: mutex) -> bool
rwlock_create() -> rwlock
rwlock_read(rwlock: rwlock) -> any
rwlock_write(rwlock: rwlock) -> bool
```

## 12. Concurrency — Async

```axl
// Future
async_spawn(future: future) -> task
async_await(task: task) -> any
async_join(tasks: list<task>) -> list<any>
async_select(futures: list<future>) -> any

// Timer
timer_after(ms: int) -> future
timer_interval(ms: int) -> stream
timer_timeout(future: future, ms: int) -> future

// Channel async
async_channel_create() -> async_channel
async_channel_send(channel: async_channel, value: any) -> future
async_channel_recv(channel: async_channel) -> future
```

## 13. Concurrency — Parallel

```axl
// Parallel iteration
parallel_map(list: list, transform: string) -> list
parallel_filter(list: list, predicate: string) -> list
parallel_for_each(list: list, action: string) -> void
parallel_reduce(list: list, reducer: string, initial: any) -> any

// Parallel scope
parallel_scope(closures: list<closure>) -> list<any>
```

## 14. System — Environment

```axl
// Environment variables
env_get(name: string) -> string
env_set(name: string, value: string) -> bool
env_list() -> map
env_exists(name: string) -> bool

// Paths
path_join(parts: list<string>) -> string
path_absolute(path: string) -> string
path_relative(path: string, base: string) -> string
path_parent(path: string) -> string
path_filename(path: string) -> string
path_extension(path: string) -> string
path_stem(path: string) -> string
path_exists(path: string) -> bool
path_is_file(path: string) -> bool
path_is_dir(path: string) -> bool

// Temp
temp_dir() -> string
temp_file() -> string
temp_create(prefix: string) -> string
```

## 15. System — Time

```axl
// Current time
time_now() -> timestamp
time_utc() -> timestamp
time_local() -> timestamp

// Components
time_year(ts: timestamp) -> int
time_month(ts: timestamp) -> int
time_day(ts: timestamp) -> int
time_hour(ts: timestamp) -> int
time_minute(ts: timestamp) -> int
time_second(ts: timestamp) -> int
time_millis(ts: timestamp) -> int

// Formatting
time_format(ts: timestamp, format: string) -> string
time_parse(text: string, format: string) -> timestamp

// Arithmetic
time_add(ts: timestamp, duration: string) -> timestamp
time_sub(a: timestamp, b: timestamp) -> duration
time_diff(a: timestamp, b: timestamp) -> int

// Sleep
time_sleep(ms: int) -> bool
time_sleep_until(ts: timestamp) -> bool
```

## 16. System — Signal

```axl
// Signal handling
signal_register(signal: string, handler: closure) -> bool
signal_unregister(signal: string) -> bool
signal_send(pid: int, signal: string) -> bool

// Process group
process_group_create() -> int
process_group_add(pid: int, group: int) -> bool
process_group_kill(group: int) -> bool
```

## 17. Database — SQL

```axl
// Connection
db_connect(url: string) -> connection
db_close(conn: connection) -> bool

// Query
db_query(conn: connection, sql: string, params: list) -> result
db_execute(conn: connection, sql: string, params: list) -> int
db_prepare(conn: connection, sql: string) -> statement
db_execute_prepared(stmt: statement, params: list) -> result

// Transaction
db_begin(conn: connection) -> transaction
db_commit(tx: transaction) -> bool
db_rollback(tx: transaction) -> bool

// Schema
db_tables(conn: connection) -> list<string>
db_columns(conn: connection, table: string) -> list<map>
db_index(conn: connection, table: string) -> list<map>
```

## 18. Database — NoSQL

```axl
// Key-Value
kv_open(path: string) -> kv_store
kv_get(store: kv_store, key: string) -> bytes
kv_set(store: kv_store, key: string, value: bytes) -> bool
kv_delete(store: kv_store, key: string) -> bool
kv_list(store: kv_store) -> list<string>
kv_iter(store: kv_store) -> stream<tuple>

// Redis
redis_connect(url: string) -> redis_conn
redis_get(conn: redis_conn, key: string) -> string
redis_set(conn: redis_conn, key: string, value: string) -> bool
redis_del(conn: redis_conn, key: string) -> bool
redis_keys(conn: redis_conn, pattern: string) -> list<string>
redis_publish(conn: redis_conn, channel: string, message: string) -> bool
redis_subscribe(conn: redis_conn, channel: string) -> stream<string>
```

## 19. Security — TLS

```axl
// TLS
tls_connect(host: string, port: int) -> tls_connection
tls_accept(listener: listener) -> tls_connection
tls_send(conn: tls_connection, data: bytes) -> bool
tls_recv(conn: tls_connection, n: int) -> bytes
tls_close(conn: tls_connection) -> bool

// Certificate
cert_load(path: string) -> certificate
cert_verify(cert: certificate, ca: certificate) -> bool
cert_info(cert: certificate) -> map
```

## 20. Graphics — Image

```axl
// Load/Save
image_load(path: string) -> image
image_save(image: image, path: string) -> bool
image_decode(data: bytes) -> image
image_encode(image: image, format: string) -> bytes

// Create
image_new(width: int, height: int, color: color) -> image
image_from_bytes(width: int, height: int, data: bytes) -> image

// Transform
image_resize(image: image, width: int, height: int) -> image
image_crop(image: image, x: int, y: int, w: int, h: int) -> image
image_rotate(image: image, degrees: int) -> image
image_flip_h(image: image) -> image
image_flip_v(image: image) -> image
image_grayscale(image: image) -> image
image_blur(image: image, radius: int) -> image
image_sharpen(image: image, amount: int) -> image

// Pixel
image_get_pixel(image: image, x: int, y: int) -> color
image_set_pixel(image: image, x: int, y: int, color: color) -> bool
image_to_bytes(image: image) -> bytes

// Composite
image_overlay(base: image, overlay: image, x: int, y: int) -> image
image_blend(a: image, b: image, opacity: int) -> image
```

## 21. Graphics — Vector/SVG

```axl
// SVG
svg_create(width: int, height: int) -> svg
svg_add_rect(svg: svg, x: int, y: int, w: int, h: int, fill: color) -> svg
svg_add_circle(svg: svg, cx: int, cy: int, r: int, fill: color) -> svg
svg_add_line(svg: svg, x1: int, y1: int, x2: int, y2: int, stroke: color) -> svg
svg_add_text(svg: svg, x: int, y: int, text: string, font: string, size: int) -> svg
svg_render(svg: svg) -> bytes
svg_to_string(svg: svg) -> string
```

## 22. Graphics — Color

```axl
// Color creation
color_rgb(r: int, g: int, b: int) -> color
color_rgba(r: int, g: int, b: int, a: int) -> color
color_hex(hex: string) -> color
color_hsl(h: int, s: int, l: int) -> color

// Color operations
color_to_hex(color: color) -> string
color_to_rgb(color: color) -> tuple
color_blend(a: color, b: color, opacity: int) -> color
color_lighten(color: color, amount: int) -> color
color_darken(color: color, amount: int) -> color
```

## 23. Graphics — PDF

```axl
// PDF
pdf_create() -> pdf
pdf_add_page(pdf: pdf) -> pdf
pdf_set_font(pdf: pdf, name: string, size: int) -> pdf
pdf_text(pdf: pdf, x: int, y: int, text: string) -> pdf
pdf_image(pdf: pdf, x: int, y: int, w: int, h: int, image: image) -> pdf
pdf_line(pdf: pdf, x1: int, y1: int, x2: int, y2: int) -> pdf
pdf_rect(pdf: pdf, x: int, y: int, w: int, h: int) -> pdf
pdf_save(pdf: pdf, path: string) -> bool
pdf_bytes(pdf: pdf) -> bytes
```

## 24. Graphics — Chart

```axl
// Chart
chart_create(type: string, data: map) -> chart
chart_set_title(chart: chart, title: string) -> chart
chart_set_labels(chart: chart, labels: list<string>) -> chart
chart_add_series(chart: chart, name: string, data: list) -> chart
chart_render(chart: chart, width: int, height: int) -> image
chart_save(chart: chart, path: string) -> bool
```

## 25. Web — Server

```axl
// HTTP Server
server_create(host: string, port: int) -> server
server_route(server: server, method: string, path: string, handler: closure) -> server
server_static(server: server, path: string, root: string) -> server
server_listen(server: server) -> bool
server_stop(server: server) -> bool

// Request/Response
request_method(req: request) -> string
request_path(req: request) -> string
request_query(req: request) -> map
request_headers(req: request) -> map
request_body(req: request) -> bytes
response_new(status: int) -> response
response_set_header(resp: response, name: string, value: string) -> response
response_set_body(resp: response, body: bytes) -> response
response_json(resp: response, value: any) -> response
response_html(resp: response, html: string) -> response
```

## 26. Web — Client

```axl
// HTTP Client
client_create() -> client
client_set_header(client: client, name: string, value: string) -> client
client_set_timeout(client: client, ms: int) -> client
client_get(client: client, url: string) -> response
client_post(client: client, url: string, body: bytes) -> response
client_put(client: client, url: string, body: bytes) -> response
client_delete(client: client, url: string) -> response
client_download(client: client, url: string, path: string) -> bool
client_upload(client: client, url: string, path: string) -> response
```

## 27. Web — HTML/CSS

```axl
// HTML
html_parse(html: string) -> document
html_query(doc: document, selector: string) -> list<element>
html_query_one(doc: document, selector: string) -> element
html_get_text(element: element) -> string
html_get_attr(element: element, name: string) -> string
html_set_attr(element: element, name: string, value: string) -> element
html_add_class(element: element, class: string) -> element
html_remove_class(element: element, class: string) -> element
html_to_string(doc: document) -> string

// CSS
css_parse(css: string) -> stylesheet
css_select(element: element, stylesheet: stylesheet) -> map
css_to_string(sheet: stylesheet) -> string
```

## 28. Web — Browser Automation

```axl
// Browser
browser_open(url: string) -> browser
browser_navigate(browser: browser, url: string) -> bool
browser_click(browser: browser, selector: string) -> bool
browser_type(browser: browser, selector: string, text: string) -> bool
browser_screenshot(browser: browser) -> image
browser_evaluate(browser: browser, js: string) -> any
browser_close(browser: browser) -> bool
```

## 29. AI — LLM

```axl
// Generation
llm_generate(provider: string, model: string, messages: list) -> string
llm_generate_stream(provider: string, model: string, messages: list) -> stream<string>
llm_generate_json(provider: string, model: string, messages: list, schema: map) -> map

// Embedding
llm_embed(provider: string, model: string, text: string) -> embedding
llm_embed_batch(provider: string, model: string, texts: list<string>) -> list<embedding>

// Similarity
llm_similarity(a: embedding, b: embedding) -> float
llm_find_similar(query: embedding, items: list<embedding>, top_k: int) -> list<int>

// Classification
llm_classify(provider: string, model: string, text: string, labels: list<string>) -> string
llm_extract(provider: string, model: string, text: string, schema: map) -> map

// Reasoning
llm_reason(provider: string, model: string, instruction: string, input: string) -> string
llm_chain_of_thought(provider: string, model: string, problem: string) -> string
```

## 30. AI — Embeddings Store

```axl
// Vector store
vector_store_create(path: string) -> vector_store
vector_store_add(store: vector_store, id: string, embedding: embedding, metadata: map) -> bool
vector_store_search(store: vector_store, query: embedding, top_k: int) -> list<result>
vector_store_delete(store: vector_store, id: string) -> bool
vector_store_count(store: vector_store) -> int
vector_store_flush(store: vector_store) -> bool
```

## 31. AI — Tokenizer

```axl
// Tokenizer
tokenizer_encode(text: string) -> list<int>
tokenizer_decode(tokens: list<int>) -> string
tokenizer_count(text: string) -> int
tokenizer_split(text: string, max_tokens: int) -> list<string>
```

## 32. FFI — Native

```axl
// Dynamic library
lib_load(path: string) -> library
lib_symbol(lib: library, name: string) -> pointer
lib_close(lib: library) -> bool

// Call
ffi_call(function: pointer, args: list<any>) -> any
ffi_call_c(name: string, library: string, args: list<any>) -> any
```

## 33. FFI — JSON-RPC

```axl
// JSON-RPC client
rpc_connect(url: string) -> rpc_client
rpc_call(client: rpc_client, method: string, params: any) -> any
rpc_notify(client: rpc_client, method: string, params: any) -> bool
rpc_close(client: rpc_client) -> bool
```

## 34. FFI — stdin/stdout

```axl
// Standard I/O
io_print(text: string) -> bool
io_println(text: string) -> bool
io_eprint(text: string) -> bool
io_eprintln(text: string) -> bool
io_readline() -> string
io_readlines() -> list<string>
io_read_all() -> string
io_read_bytes(n: int) -> bytes
```

## 35. System — Archive

```axl
// ZIP
zip_create(path: string) -> archive
zip_add(archive: archive, name: string, data: bytes) -> archive
zip_extract(archive: archive, dest: string) -> bool
zip_list(archive: archive) -> list<string>

// Tar
tar_create(path: string) -> archive
tar_add(archive: archive, name: string, data: bytes) -> archive
tar_extract(archive: archive, dest: string) -> bool

// Gzip
gzip_compress(data: bytes) -> bytes
gzip_decompress(data: bytes) -> bytes
```

## 36. System — Cron/Scheduler

```axl
// Scheduler
scheduler_create() -> scheduler
scheduler_schedule(scheduler: scheduler, cron: string, task: closure) -> job
scheduler_cancel(scheduler: scheduler, job: job) -> bool
scheduler_start(scheduler: scheduler) -> bool
scheduler_stop(scheduler: scheduler) -> bool
```

## 37. System — Logging

```axl
// Logger
log_info(message: string) -> bool
log_warn(message: string) -> bool
log_error(message: string) -> bool
log_debug(message: string) -> bool
log_trace(message: string) -> bool
log_set_level(level: string) -> bool
log_set_output(path: string) -> bool
```

## 38. System — Metrics

```axl
// Metrics
metric_counter(name: string, value: int) -> bool
metric_gauge(name: string, value: int) -> bool
metric_histogram(name: string, value: int) -> bool
metric_timer_start(name: string) -> timer
metric_timer_stop(timer: timer) -> int
```

## 39. System — Cache

```axl
// Cache
cache_create(max_size: int) -> cache
cache_get(cache: cache, key: string) -> any
cache_set(cache: cache, key: string, value: any, ttl_ms: int) -> bool
cache_delete(cache: cache, key: string) -> bool
cache_clear(cache: cache) -> bool
cache_size(cache: cache) -> int
```

## 40. System — Rate Limiter

```axl
// Rate limiter
ratelimit_create(max: int, window_ms: int) -> ratelimiter
ratelimit_acquire(rl: ratelimiter) -> bool
ratelimit_release(rl: ratelimiter) -> bool
ratelimit_available(rl: ratelimiter) -> int
```

## 41. System — Retry

```axl
// Retry
retryExecute(closure: closure, max_retries: int, delay_ms: int) -> any
retryExecuteWithBackoff(closure: closure, max_retries: int, base_delay_ms: int) -> any
```

## 42. System — Config

```axl
// Config
config_load(path: string) -> map
config_get(config: map, key: string) -> any
config_set(config: map, key: string, value: any) -> map
config_save(config: map, path: string) -> bool
config_merge(base: map, overlay: map) -> map
```

## 43. System — Validation

```axl
// Validate
validate_email(email: string) -> bool
validate_url(url: string) -> bool
validate_ip(ip: string) -> bool
validate_uuid(uuid: string) -> bool
validate_json(text: string) -> bool
validate_regex(text: string, pattern: string) -> bool
validate_schema(value: any, schema: map) -> bool
```

## 44. System — UUID

```axl
// UUID
uuid_v4() -> string
uuid_v5(namespace: string, name: string) -> string
uuid_parse(text: string) -> uuid
uuid_to_string(uuid: uuid) -> string
uuid_validate(text: string) -> bool
```

## 45. System — Hash Table

```axl
// Hash map (persistent)
hashmap_create() -> hashmap
hashmap_get(map: hashmap, key: string) -> any
hashmap_set(map: hashmap, key: string, value: any) -> hashmap
hashmap_delete(map: hashmap, key: string) -> hashmap
hashmap_contains(map: hashmap, key: string) -> bool
hashmap_size(map: hashmap) -> int
hashmap_keys(map: hashmap) -> list<string>
hashmap_values(map: hashmap) -> list
hashmap_clear(map: hashmap) -> hashmap
```

## 46. System — Bloom Filter

```axl
// Bloom filter
bloom_create(expected_items: int, false_positive_rate: float) -> bloom
bloom_add(bloom: bloom, item: string) -> bloom
bloom_contains(bloom: bloom, item: string) -> bool
bloom_count(bloom: bloom) -> int
```

## 47. System — Trie

```axl
// Trie
trie_create() -> trie
trie_insert(trie: trie, word: string) -> trie
trie_search(trie: trie, word: string) -> bool
trie_starts_with(trie: trie, prefix: string) -> bool
trie_words(trie: trie, prefix: string) -> list<string>
```

## 48. System — Graph

```axl
// Graph
graph_create() -> graph
graph_add_node(graph: graph, id: string, data: any) -> graph
graph_add_edge(graph: graph, from: string, to: string, weight: int) -> graph
graph_remove_node(graph: graph, id: string) -> graph
graph_remove_edge(graph: graph, from: string, to: string) -> graph
graph_nodes(graph: graph) -> list<string>
graph_edges(graph: graph) -> list<tuple>
graph_neighbors(graph: graph, id: string) -> list<string>
graph_bfs(graph: graph, start: string) -> list<string>
graph_dfs(graph: graph, start: string) -> list<string>
graph_dijkstra(graph: graph, start: string, end: string) -> list<string>
graph_topological_sort(graph: graph) -> list<string>
graph_cycle(graph: graph) -> bool
```

## 49. System — Tree

```axl
// Tree
tree_create(root: any) -> tree
tree_add_child(tree: tree, parent: string, child: any) -> tree
tree_remove(tree: tree, id: string) -> tree
tree_find(tree: tree, id: string) -> node
tree_depth(tree: tree) -> int
tree_breadth(tree: tree) -> int
tree_preorder(tree: tree) -> list
tree_postorder(tree: tree) -> list
tree_level_order(tree: tree) -> list
```

## 50. System — State Machine

```axl
// FSM
fsm_create(initial: string) -> fsm
fsm_add_state(fsm: fsm, name: string) -> fsm
fsm_add_transition(fsm: fsm, from: string, to: string, event: string) -> fsm
fsm_trigger(fsm: fsm, event: string) -> string
fsm_state(fsm: fsm) -> string
fsm_history(fsm: fsm) -> list<string>
```

---

## Total: 50 categories, 500+ primitives

Every primitive is:
- **Composable**: Returns values other primitives consume
- **Type-safe**: Clear input/output types
- **Agent-native**: Optimized for LLM generation
- **Safe**: Wrapped in AXL's permission system
- **Rust-backed**: Each maps to safe Rust code
