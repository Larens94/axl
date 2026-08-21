# Primitiva Native

90+ primitiva che wrappano le capability fondamentali di Rust. Gli agenti AXL le combinano per costruire qualsiasi software.

## Sintassi

```axl
2;10|result|"hello",!text_upper/1|s;12|$result
```

`!nome/primitiva/arietà` chiama una primitiva nativa. Gli argomenti sono sullo stack RPN.

## I/O — File

```axl
!file_read/1       # legge un file
!file_write/2      # scrive un file
!file_exists/1     # controlla esistenza
!file_size/1       # dimensione in byte
!file_delete/1     # elimina un file
!file_copy/2       # copia file
!file_move/2       # rinomina/sposta
!dir_create/1      # crea directory
!dir_list/1        # elenca contenuto
!dir_delete/1      # elimina directory
```

## Text

```axl
!text_upper/1      # maiuscolo
!text_lower/1      # minuscolo
!text_trim/1       # rimuove spazi
!text_replace/3    # sostituisce testo
!text_split/2      # split per delimitatore
!text_join/2       # join con delimitatore
!text_find/2       # trova indici
!text_contains/2   # contiene pattern
!text_matches/2    # regex match
!text_length/1     # lunghezza
!text_reverse/1    # inverte
!text_lines/1      # righe
!text_extract/2    # estrae con regex
```

## Collections — List

```axl
!list_new/0        # lista vuota
!list_push/2       # aggiunge elemento
!list_length/1     # lunghezza
!list_contains/2   # contiene
!list_sort/1       # ordina
!list_reverse/1    # inverte
!list_unique/1     # unici
!list_flatten/1    # appiattisce
!list_slice/3      # sotto-lista
!list_head/1       # primo elemento
!list_tail/1       # tutti tranne primo
!list_sum/1        # somma numeri
!list_diff/2       # differenza
```

## Collections — Map

```axl
!map_new/0         # mappa vuota
!map_get/2         # ottiene valore
!map_set/3         # imposta valore
!map_keys/1        # chiavi
!map_values/1      # valori
!map_contains/2    # contiene chiave
!map_delete/2      # rimuove chiave
!map_merge/2       # merge due mappe
!map_entries/1     # tuple chiave-valore
```

## Math

```axl
!math_add/2        # a + b
!math_sub/2        # a - b
!math_mul/2        # a * b
!math_div/2        # a / b
!math_mod/2        # a % b
!math_pow/2        # a ^ b
!math_abs/1        # valore assoluto
!math_min/2        # minimo
!math_max/2        # massimo
!math_clamp/3      # limita range
!math_random/0     # casuale
!math_random_range/2 # range casuale
!math_sum/1        # somma lista
!math_average/1    # media lista
```

## Crypto

```axl
!hash_sha256/1     # hash SHA-256
!hash_blake3/1     # hash BLAKE3
!hash_md5/1        # hash MD5
!encode_base64/1   # codifica Base64
!decode_base64/1   # decodifica Base64
!encode_hex/1      # codifica Hex
!decode_hex/1      # decodifica Hex
!crypto_random_bytes/1 # bytes casuali
```

## JSON

```axl
!json_parse/1      # parse JSON
!json_stringify/1  # stringify JSON
!json_validate/1   # valida JSON
```

## Network

```axl
!http_get/1        # HTTP GET
!http_post/2       # HTTP POST
```

## System

```axl
!env_get/1         # variabile ambiente
!env_list/0        # tutte le variabili
!time_now/0        # timestamp ms
!time_format/2     # formatta timestamp
!time_sleep/1      # dorme N ms
!path_join/1       # unisce path
!path_absolute/1   # path assoluto
!path_parent/1     # directory padre
!path_filename/1   # nome file
!path_extension/1  # estensione
!path_exists/1     # esiste
!temp_dir/0        # directory temp
!temp_file/0       # file temp
!sys_hostname/0    # hostname
!sys_os/0          # sistema operativo
!sys_arch/0        # architettura
!process_run/1     # esegue comando
!process_output/1  # output comando
```

## Test

```axl
# Text upper
2;10|r|"hello",!text_upper/1|s;12|$r
# → "HELLO"

# JSON parse
2;10|d|'{"x":42}',!json_parse/1|s;12|$d
# → {"x": 42}

# SHA-256 hash
2;10|h|"test",!hash_sha256/1|s;12|$h
# → "9f86d08..."
```
