# AXL 3.0 — Analisi Elementi Mancanti

## Status Attuale (216 primitive)

| Categoria | Primitive | Stato |
|---|---|---|
| I/O File | 10 | Completo |
| Text | 13 | Completo |
| Collections | 17 | Completo |
| Math | 14 | Completo |
| System | 18 | Completo |
| JSON | 3 | Completo |
| Crypto | 8 | Completo |
| HTTP Client | 6 | Completo |
| HTTP Server | 12 | Funzionante |
| Database | 12 | Funzionante |
| WebSocket | 6 | Stub |
| **Totale** | **216** | |

---

## 1. HTTP SERVER (Implementare per primo)

### Cosa serve
Un server HTTP che accetta richieste e risponde.

### Primitive da implementare

```axl
# Creare server
!http_server_create/1,"8080"           → server_id

# Route handling
!http_server_route/4,$server,"GET","/api/users",handler
!http_server_static/3,$server,"/","./public"

# Avviare il server
!http_server_listen/1,$server          → blocca

# Response helpers
!http_response/2,#200,"body"           → response
!http_response_json/2,#200,$data       → response
!http_response_html/2,#200,"<html>..."  → response
!http_response_error/2,#404,"not found" → response

# Request parsing
!http_request_method/1,$request        → "GET"
!http_request_path/1,$request          → "/api/users"
!http_request_query/1,$request         → map
!http_request_body/1,$request          → string
!http_request_header/2,$request,"auth" → string
```

### Perché è fondamentale
Senza HTTP server non possiamo:
- Servire frontend web
- Creare API REST
- Costruire web application

---

## 2. DATABASE (Implementare per secondo)

### Cosa serve
Persistenza dati con SQL.

### Primitive da implementare

```axl
# Connettere
!db_connect/1,"/path/to/db.sqlite"     → db_id
!db_connect/1,":memory:"               → db_id (in-memory)

# Esecuzione SQL
!db_execute/2,$db_id,"CREATE TABLE..."  → result
!db_query/2,$db_id,"SELECT * FROM..."   → rows
!db_prepare/2,$db_id,"INSERT INTO..."   → statement

# Transazioni
!db_begin/1,$db_id                     → tx_id
!db_commit/1,$tx_id                    → bool
!db_rollback/1,$tx_id                  → bool

# Utility
!db_tables/1,$db_id                    → list
!db_columns/2,$db_id,"users"           → list
!db_count/2,$db_id,"users"             → int

# Prepared statements
!db_execute_prepared/3,$stmt,$params    → result
!db_query_prepared/3,$stmt,$params     → rows
```

### Perché è fondamentale
Senza database non possiamo:
- Persistere dati utente
- Implementare CRUD
- Gestire sessioni
- Cache persistente

---

## 3. WEBSOCKET (Implementare per terzo)

### Cosa serve
Comunicazione real-time bidirezionale.

### Primitive da implementare

```axl
# Server WebSocket
!ws_server_create/1,"8080"             → server_id
!ws_server_listen/1,$server            → blocca
!ws_server_broadcast/2,$server,$data   → bool

# Client WebSocket
!ws_connect/1,"ws://localhost:8080"    → conn_id
!ws_send/2,$conn_id,$data             → bool
!ws_recv/1,$conn_id                   → message
!ws_close/1,$conn_id                  → bool

# Event handling
!ws_on_message/2,$server,$handler     → bool
!ws_on_connect/2,$server,$handler     → bool
!ws_on_disconnect/2,$server,$handler  → bool
```

### Perché è fondamentale
Senza WebSocket non possiamo:
- Chat real-time
- Notifiche push
- Collaborative editing
- Live updates

---

## 4. AUTENTICAZIONE

### Cosa serve
Gestione utenti e sicurezza.

### Primitive da implementare

```axl
# Password
!auth_hash_password/1,"password"       → hash
!auth_verify_password/2,"password",$hash → bool

# JWT
!auth_jwt_create/2,$payload,$secret    → token
!auth_jwt_verify/2,$token,$secret      → payload
!auth_jwt_decode/1,$token              → payload

# Sessioni
!session_create/1,$user_data           → session_id
!session_get/1,$session_id             → session_data
!session_destroy/1,$session_id         → bool

# OAuth
!oauth_authorize/2,$provider,$scopes   → url
!oauth_token/3,$provider,$code,$secret → token
```

---

## 5. EMAIL

### Primitive

```axl
# Invio email
!email_send/4,$to,$subject,$body,$from → bool
!email_send_html/4,$to,$subject,$html,$from → bool
!email_send_attach/5,$to,$subject,$body,$attachments → bool

# Template
!email_template/2,$template,$data     → html
```

---

## 6. CACHE

### Primitive

```axl
# Cache in-memory
!cache_create/1,#1000                  → cache_id (max entries)
!cache_get/2,$cache_id,"key"           → value
!cache_set/3,$cache_id,"key",$value    → bool
!cache_set_ttl/4,$cache_id,"key",$value,#3600 → bool
!cache_delete/2,$cache_id,"key"        → bool
!cache_clear/1,$cache_id               → bool
!cache_size/1,$cache_id                → int
```

---

## 7. RATE LIMITING

### Primitive

```axl
!ratelimit_create/2,#100,#60000        → limiter_id (100 req/60s)
!ratelimit_check/2,$limiter_id,"ip"    → bool (allowed?)
!ratelimit_reset/1,$limiter_id         → bool
```

---

## 8. LOGGING

### Primitive

```axl
!log_info/1,"message"                  → bool
!log_warn/1,"message"                  → bool
!log_error/1,"message"                 → bool
!log_debug/1,"message"                 → bool
!log_set_level/1,"debug"               → bool
!log_set_file/1,"/var/log/app.log"     → bool
!log_json/2,"event",$data              → bool
```

---

## 9. METRICS / OBSERVABILITY

### Primitive

```axl
!metric_counter/2,"requests_total",#1  → bool
!metric_gauge/2,"connections",#42      → bool
!metric_histogram/2,"latency_ms",#150  → bool
!metric_timer_start/1,"operation"      → timer_id
!metric_timer_stop/1,$timer_id         → duration_ms
```

---

## 10. TEMPLATE ENGINE

### Primitive

```axl
!template_render/2,"Hello {{name}}",{"name":"World"} → string
!template_render_file/2,"/templates/page.html",$data → html
!template_escape/1,"<script>"          → escaped_string
```

---

## 11. PDF GENERATION

### Primitive

```axl
!pdf_create/0                          → pdf_id
!pdf_set_font/3,$pdf,"Arial",#12      → pdf
!pdf_text/4,$pdf,#100,#100,"Hello"    → pdf
!pdf_image/5,$pdf,#100,#100,#200,#150,$image → pdf
!pdf_line/5,$pdf,#0,#0,#100,#100      → pdf
!pdf_rect/5,$pdf,#0,#0,#100,#50       → pdf
!pdf_save/2,$pdf,"output.pdf"          → bool
!pdf_bytes/1,$pdf                      → bytes
```

---

## 12. IMAGE PROCESSING

### Primitive

```axl
!image_load/1,"photo.jpg"              → image_id
!image_save/2,$image_id,"output.jpg"   → bool
!image_resize/3,$image_id,#800,#600    → image_id
!image_crop/5,$image_id,#0,#0,#400,#300 → image_id
!image_rotate/2,$image_id,#90          → image_id
!image_grayscale/1,$image_id           → image_id
!image_blur/2,$image_id,#5             → image_id
!image_get_pixel/3,$image_id,#100,#50  → color
!image_set_pixel/4,$image_id,#100,#50,$color → bool
!image_to_bytes/1,$image_id            → bytes
!image_from_bytes/3,$width,$height,$bytes → image_id
```

---

## 13. FILE SYSTEM WATCHERS

### Primitive

```axl
!watch_create/1,"/path/to/dir"         → watcher_id
!watch_add/2,$watcher_id,"*.txt"       → bool
!watch_remove/2,$watcher_id,"*.txt"    → bool
!watch_poll/1,$watcher_id              → events (list)
!watch_close/1,$watcher_id             → bool
```

---

## 14. CRON / SCHEDULER

### Primitive

```axl
!cron_create/0                         → cron_id
!cron_add/3,$cron_id,"*/5 * * * *",$task → job_id
!cron_remove/2,$cron_id,$job_id       → bool
!cron_start/1,$cron_id                → bool
!cron_stop/1,$cron_id                 → bool
!cron_list/1,$cron_id                 → list<job>
```

---

## 15. SECRETS MANAGEMENT

### Primitive

```axl
!secret_store/2,"api_key","sk-xxx"     → bool
!secret_get/1,"api_key"               → string
!secret_delete/1,"api_key"            → bool
!secret_list/0                        → list
```

---

## 16. PROCESS MANAGEMENT (Migliorare)

### Primitive attuali
```axl
!process_run/1,"command"               → output
!process_output/1,"command"            → output
```

### Da aggiungere
```axl
!process_spawn/1,"command"             → pid
!process_status/1,$pid                 → status
!process_kill/1,$pid                   → bool
!process_wait/1,$pid                   → exit_code
!process_pipe/1,"command"              → stream
```

---

## 17. NETWORK (Migliorare)

### Primitive attuali
```axl
!http_get/1,$url                       → response
!http_post/2,$url,$body               → response
```

### Da aggiungere
```axl
!http_put/2,$url,$body                → response
!http_delete/1,$url                   → response
!http_patch/2,$url,$body              → response
!http_request/4,$method,$url,$headers,$body → response
!http_download/2,$url,$path           → bool
!http_upload/3,$url,$path,$field      → response
```

---

## 18. COMPRESSIONE

### Primitive

```axl
!gzip_compress/1,$data                → bytes
!gzip_decompress/1,$data              → bytes
!zstd_compress/1,$data                → bytes
!zstd_decompress/1,$data              → bytes
!brotli_compress/1,$data              → bytes
!brotli_decompress/1,$data            → bytes
```

---

## 19. REGEX (Migliorare)

### Primitive attuali
```axl
!text_matches/2,$text,$regex          → bool
!text_find/2,$text,$regex             → list
!text_extract/2,$text,$regex          → list
```

### Da aggiungere
```axl
!regex_replace/3,$text,$pattern,$replacement → string
!regex_split/2,$text,$pattern         → list
!regex_captures/2,$text,$pattern      → list<map>
```

---

## 20. DATE/TIME (Migliorare)

### Primitive attuali
```axl
!time_now/0                           → timestamp
!time_format/2,$timestamp,$format     → string
!time_sleep/1,$ms                     → bool
```

### Da aggiungere
```axl
!time_parse/2,$string,$format         → timestamp
!time_add/2,$timestamp,$duration      → timestamp
!time_sub/2,$a,$b                     → duration_millis
!time_diff/2,$a,$b                    → duration_millis
!time_year/1,$timestamp               → int
!time_month/1,$timestamp              → int
!time_day/1,$timestamp                → int
!time_hour/1,$timestamp               → int
!time_minute/1,$timestamp             → int
!time_second/1,$timestamp             → int
```

---

## 21. VALIDATION

### Primitive

```axl
!validate_email/1,"user@example.com"   → bool
!validate_url/1,"https://example.com" → bool
!validate_ip/1,"192.168.1.1"          → bool
!validate_uuid/1,"550e8400-..."       → bool
!validate_json/1,"{...}"              → bool
!validate_regex/2,$text,$pattern      → bool
!validate_credit_card/1,"4111..."     → bool
!validate_phone/1,"+1234567890"       → bool
```

---

## 22. UUID

### Primitive

```axl
!uuid_v4/0                            → string
!uuid_v5/2,$namespace,$name           → string
!uuid_parse/1,"550e8400-..."          → bool
!uuid_validate/1,"550e8400-..."       → bool
```

---

## 23. HASH TABLE (Persistente)

### Primitive

```axl
!hashmap_open/1,"path.db"             → hashmap_id
!hashmap_get/2,$hashmap_id,"key"      → value
!hashmap_set/3,$hashmap_id,"key",$value → bool
!hashmap_delete/2,$hashmap_id,"key"   → bool
!hashmap_keys/1,$hashmap_id           → list
!hashmap_values/1,$hashmap_id         → list
!hashmap_size/1,$hashmap_id           → int
!hashmap_clear/1,$hashmap_id          → bool
!hashmap_close/1,$hashmap_id          → bool
```

---

## 24. GRAPH

### Primitive

```axl
!graph_create/0                       → graph_id
!graph_add_node/3,$graph_id,"id",$data → bool
!graph_add_edge/4,$graph_id,"from","to",#1 → bool
!graph_remove_node/2,$graph_id,"id"   → bool
!graph_remove_edge/3,$graph_id,"from","to" → bool
!graph_nodes/1,$graph_id              → list
!graph_edges/1,$graph_id              → list
!graph_neighbors/2,$graph_id,"id"     → list
!graph_bfs/2,$graph_id,"start"        → list
!graph_dfs/2,$graph_id,"start"        → list
!graph_dijkstra/3,$graph_id,"start","end" → list
!graph_topological/1,$graph_id        → list
!graph_has_cycle/1,$graph_id          → bool
```

---

## 25. TREE

### Primitive

```axl
!tree_create/1,$root_data             → tree_id
!tree_add_child/3,$tree_id,"parent_id",$data → bool
!tree_remove/2,$tree_id,"id"          → bool
!tree_find/2,$tree_id,"id"            → node
!tree_depth/1,$tree_id                → int
!tree_breadth/1,$tree_id              → int
!tree_preorder/1,$tree_id             → list
!tree_postorder/1,$tree_id            → list
!tree_level_order/1,$tree_id          → list
```

---

## 26. STATE MACHINE

### Primitive

```axl
!fsm_create/1,"idle"                  → fsm_id
!fsm_add_state/2,$fsm_id,"running"    → bool
!fsm_add_transition/4,$fsm_id,"idle","running","start" → bool
!fsm_trigger/2,$fsm_id,"start"        → new_state
!fsm_state/1,$fsm_id                  → state
!fsm_history/1,$fsm_id                → list
```

---

## 27. BLOOM FILTER

### Primitive

```axl
!bloom_create/2,#1000,0.01            → bloom_id (1000 items, 1% FP)
!bloom_add/2,$bloom_id,"item"         → bool
!bloom_check/2,$bloom_id,"item"       → bool
!bloom_count/1,$bloom_id              → int
```

---

## 28. TRIE

### Primitive

```axl
!trie_create/0                        → trie_id
!trie_insert/2,$trie_id,"word"        → bool
!trie_search/2,$trie_id,"word"        → bool
!trie_starts_with/2,$trie_id,"prefix" → bool
!trie_words/2,$trie_id,"prefix"       → list
```

---

## 29. EMBEDDINGS / VECTOR

### Primitive

```axl
!embedding_create/1,$text             → embedding
!embedding_similarity/2,$a,$b         → float
!embedding_distance/2,$a,$b           → float
!embedding_dimensions/1,$embedding    → int
```

---

## 30. ENCODING

### Primitive (estendere)
```axl
!encode_url/1,"hello world"           → "hello%20world"
!decode_url/1,"hello%20world"         → "hello world"
!encode_html/1,"<script>"             → "&lt;script&gt;"
!decode_html/1,"&lt;script&gt;"        → "<script>"
!encode_xml/1,"<tag>"                 → escaped_xml
!decode_xml/1,$xml                    → string
```

---

## RIEPILOGO PRIORITÀ

### Priorità 1 (Fondamentale per applicazioni web)
1. ✅ HTTP Client (6 primitive: GET, POST, PUT, DELETE, PATCH, download)
2. ✅ HTTP Server (12 primitive: create, route, static, listen, response, request parsing)
3. ✅ Database SQLite (12 primitive: connect, execute, query, CRUD, transactions, schema)
4. 🔲 WebSocket (stub → implementare)

### Priorità 2 (Autenticazione e Sicurezza)
5. 🔲 Authentication (JWT, password, session)
6. 🔲 Cache (in-memory con TTL)

### Priorità 3 (Comunicazione)
7. 🔲 Email
8. 🔲 Rate Limiting

### Priorità 4 (Data Processing)
9. 🔲 Compressione
10. 🔲 Template Engine
11. 🔲 PDF Generation
12. 🔲 Image Processing

### Priorità 5 (Scheduling e Monitoring)
13. 🔲 Cron/Scheduler
14. 🔲 Logging strutturato
15. 🔲 Metrics/Observability

### Priorità 6 (Data Structures)
16. 🔲 Hash Table persistente
17. 🔲 Graph
18. 🔲 Tree
19. 🔲 State Machine
20. 🔲 Bloom Filter
21. 🔲 Trie

### Priorità 7 (Utility)
22. 🔲 Validation (email, URL, UUID)
23. 🔲 UUID generation
24. 🔲 Regex avanzato
25. 🔲 Date/Time esteso

### Priorità 8 (File System)
26. 🔲 File Watchers
27. 🔲 Clipboard
28. 🔲 Notifications

### Priorità 9 (Network Esteso)
29. 🔲 HTTP PUT/DELETE/PATCH
30. 🔲 File download/upload

### Priorità 10 (Speciali)
31. 🔲 Secrets Management
32. 🔲 Embeddings/Vector
33. 🔲 Process Management esteso
34. 🔲 Compressione avanzata
35. 🔲 Encoding avanzato

---

## TOTALE: 35 primitive mancanti per avere un linguaggio completo

### Impatto

| Categoria | Primitive | Impatto |
|---|---|---|
| HTTP Server | 12 | **CRITICO** — serve web app |
| Database | 7 | **CRITICO** — persistenza dati |
| WebSocket | 6 | **ALTO** — real-time |
| Authentication | 6 | **ALTO** — sicurezza |
| Cache | 6 | **ALTO** — performance |
| Email | 4 | **MEDIO** — comunicazione |
| Rate Limiting | 3 | **MEDIO** — sicurezza |
| Template | 3 | **MEDIO** — frontend |
| PDF | 8 | **BASSO** — documenti |
| Image | 12 | **BASSO** — media |
| File Watch | 5 | **BASSO** — automation |
| Cron | 5 | **BASSO** — scheduling |
| Logging | 6 | **BASSO** — monitoring |
| Metrics | 5 | **BASSO** — observability |
| Data Structures | 30 | **BASSO** — advanced |
| Validation | 8 | **BASSO** — utility |
| UUID | 4 | **BASSO** — utility |
| Regex | 3 | **BASSO** — utility |
| Date/Time | 10 | **BASSO** — utility |
