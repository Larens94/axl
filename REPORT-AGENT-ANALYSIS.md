# Report: AXL come Linguaggio per Modelli/Agenti LLM

## Executive Summary

AXL è **ottimamente posizionato** come linguaggio per agenti LLM, con alcune criticità da risolvere. Il vantaggio principale è la compattità token + determinismo parsing. La criticità principale è la mancanza di un parser keyword-based per l'editing umano.

---

## 1. Token Efficiency (10/10)

### Confronto

| Task | Python | AXL | Risparmio |
|------|--------|-----|-----------|
| Read file | `open('f.txt').read()` (11 tokens) | `"f.txt",!file_read/1` (5 tokens) | 55% |
| Hash password | `hashlib.sha256(p.encode()).hexdigest()` (15 tokens) | `"pass",!auth_hash_password/1` (5 tokens) | 67% |
| HTTP GET | `requests.get(url).text` (8 tokens) | `url,!http_get/1` (4 tokens) | 50% |
| Classify text | `classify(text, labels)` (4 tokens) | `text,"labels",!llm_classify/3` (6 tokens) | -50%* |

*Le primitiva LLM richiedono più token perché trasportano istruzioni.

### Verdetto
AXL è **estremamente efficiente** per operazioni I/O, data, e sistema. Per operazioni LLM il risparmio è minore perché le istruzioni di reasoning richiedono token.

---

## 2. Determinismo Parsing (9/10)

### Punti di forza
- Formato RPN non ambiguo
- Opcode numerici fissi
- Nessuna dipendenza da contesto
- Parsing in O(n)

### Criticità
- Se l'LLM genera un token sbagliato (es. `!text_upper/2` invece di `/1`), il parser fallisce silenziosamente
- Non c'è auto-correzione: errore = programma non eseguibile

### Soluzione
Aggiungere un **fallback parser** che prova a correggere errori comuni:
- Arità sbagliata → prova arità corretta
- Opcode sconosciuto → suggerisce simili
- Argomenti mancanti → inserisce default

---

## 3. Composabilità (8/10)

### Cosa funziona
```axl
# Pipeline componibile
10|step1|input,!primitiva_a/1|s;
10|step2|$step1,!primitiva_b/2,"arg"|s;
12|$step2
```

### Criticità
- Le chiamate annidate sono complesse in RPN
- Non ci sono pattern matching nativi
- Le strutture dati composte (mappe annidate) sono verbose

### Soluzione
- Aggiungere macro/sintassi abbreviata per pipeline
- Supportare destructuring

---

## 4. Espressività per Agenti (7/10)

### Cosa AXL fa bene
- I/O file/rete
- Manipolazione dati
- Crittografia
- Database
- Validazione

### Cosa manca per agenti
- **Reasoning strutturato** — Chain-of-thought è una stringa, non un albero
- **Memoria semantica** — Gli embedding sono vettori grezzi, non un grafo
- **Comunicazione inter-agenti** — Messaggi sono stringhe, non protocolli
- **Scheduling** — Non c'è un scheduler nativo per agenti periodici

---

## 5. Error Handling (6/10)

### Situazione attuale
- `PrimitiveError` ritorna una stringa
- Nessun tipo di errore strutturato
- L'LLM non può distinguere tra errori recuperabili e fatali

### Soluzione
- Errori strutturati con codici
- Retry automatico per errori di rete
- Fallback per primitive fallite

---

## 6. Learning Curve per LLM (9/10)

### Vantaggi
- Sintassi compatta = pochi token da imparare
- Opcode fissi = pattern ripetibili
- Esempi nel training data = facile da generalizzare

### Criticità
- Formato non standard (non è Python/JS)
- Poche risorse di training su AXL
- Errori di sintassi difficili da correggere senza feedback

---

## 7. Confronto con Alternative

| Aspetto | Python/LangChain | AXL |
|---------|-----------------|-----|
| Token efficiency | Bassa | Alta |
| Determinismo parsing | Dipende da LLM | Alto |
| Esecuzione | Interpretata | Native (Rust) |
| Performance | Bassa | Alta |
| Ecosistema | Grande | In costruzione |
| Debugging | Facile | Difficile |
| Composabilità | Alta | Media |
| Sicurezza | Bassa | Alta |

---

## 8. Raccomandazioni

### Priorità 1 (Fondamentali)
1. **Keyword parser** — Per debugging umano e testing
2. **Errori strutturati** — Codici di errore, non solo stringhe
3. **Auto-correzione** — Parser che corregge errori comuni

### Priorità 2 (Importanti)
4. **Reasoning strutturato** — Albero di ragionamento, non stringa
5. **Pattern matching** — Destructuring per strutture dati
6. **Pipeline operator** — Composizione fluida di primitiva

### Priorità 3 (Utili)
7. **LSP** — Language Server Protocol per IDE
8. **Formatter** — Formattazione automatica
9. **Linter** — Controllo errori statico
10. **Debugger** — Step-by-step execution

---

## 9. Verdetto Finale

### AXL come linguaggio per agenti: **7.5/10**

**Punti di forza:**
- Token efficiency eccezionale
- Determinismo parsing alto
- Performance native (Rust)
- Sicurezza (type-safe, sandbox)
- 194 primitiva complete

**Punti deboli:**
- Parser keyword mancante (debugging difficile)
- Error handling non strutturato
- Reasoning non componibile
- Ecosistema in costruzione
- Pochi tool per sviluppo (LSP, formatter, debugger)

**Potenziale:**
- Con keyword parser + error handling strutturati → **9/10**
- Con ecosistema maturo → **9.5/10**
- Con debugging tools → **10/10**

---

## 10. Conclusione

AXL è un **ottimo linguaggio per agenti LLM** con un vantaggio competitivo chiaro: **token efficiency + determinismo + performance native**. Le criticità sono tutte risolvibili con sviluppo incrementale.

Il vantaggio rispetto a Python/LangChain è significativo per:
- Agenti in produzione (performance + sicurezza)
- Agenti con molte tool call (token efficiency)
- Agenti che devono essere affidabili (determinismo)

Lo svantaggio è per:
- Sviluppo e debugging (manca keyword parser)
- Prototyping rapido (Python è più veloce)
- Community e risorse (Python ha più tooling)
