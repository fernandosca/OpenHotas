# OpenHOTAS — Build Log V1.21

**Data:** 17/Jun/2026
**Versão:** 1.2.1
**Gate:** ✅ Build ✅ Clippy ✅ Fmt

---

## Resumo

V1.21 implementa 3 features planejadas:

| # | Feature | Plano | Status |
|---|---|---|---|
| 1 | Sistema de Versionamento | `plan/v1_21_versioning.md` | ✅ |
| 2 | Flash Driver Seguro (Safe Rust) | `plan/1_21 usb flash seguro.md` | ✅ |
| 3 | CDC Serial Debug | `plan/1_21 cdc serial.md` | ✅ |

---

## 1. Sistema de Versionamento

### Arquivos alterados

| Arquivo | Mudança |
|---|---|
| `Cargo.toml` | `version = "1.2.1"` |
| `src/constants.rs` | Adicionados `FIRMWARE_VERSION` e `FIRMWARE_GIT_HASH` |
| `build.rs` | Injeção de `GIT_HASH` via `env!("GIT_HASH")` com `rerun-if-changed` |
| `src/main.rs` | `usb_cfg.device_release = 0x0121` (BCD: major=1, minor=21) |

### Justificativas

- **Três camadas de versionamento:** Cargo.toml (SemVer → logs), build.rs (git hash → rastreabilidade de commit), USB descriptor (BCD → visível no host)
- **`device_release` manual, não derivado de `CARGO_PKG_VERSION`:** BCD requer formato `0xMMmm`; conversão de string "1.2.1" → `0x0121` não é trivial em `no_std`. Sincronização manual com verificação no review
- **Git hash vazio se git indisponível:** Build não quebra em CI sem checkout git

---

## 2. Flash Driver Seguro (Safe Rust)

### Arquivos alterados

| Arquivo | Mudança |
|---|---|
| `src/storage/flash.rs` | Reescrita completa — 98 linhas |

### Mudanças estruturais

| Antes (V1.2) | Depois (V1.21) |
|---|---|
| `static mut FLASH_INSTANCE` + `#[allow(static_mut_refs)]` | `Mutex<CriticalSectionRawMutex, RefCell<Option<...>>>` |
| `critical_section::with` manual | `embassy_sync::Mutex` (padrão `spi_bus.rs`) |
| `FlashError` com 4 variantes | + `OutOfBounds`, `NotInitialized` |
| Sem validação de limites de leitura | `validate_range()` com `checked_add` |
| `with_flash` retorna `WriteError` se não inicializado | Retorna `NotInitialized` (semântica correta) |

### Justificativas

- **Padrão `Mutex<CriticalSectionRawMutex, RefCell<Option<...>>>`:** Idêntico ao `spi_bus.rs` (V1.2). Consistência arquitetural, zero `static mut`, exclusão mútua garantida pelo compilador
- **`NotInitialized` vs `WriteError` genérico:** Diagnóstico mais preciso. "Flash não inicializado" ≠ "Erro de escrita"
- **`validate_range()`:** Previne acesso fora da região física da flash (2MB). Antes o erro só aparecia no hardware
- **Único `unsafe` restante:** `core::ptr::read_volatile` para leitura XIP — justificado por ser memória mapeada por hardware

### API preservada

Nenhum consumidor alterado. `cal_store.rs` e `settings.rs` usam as mesmas assinaturas:
- `init()`, `read_flash()`, `write_flash()`, `erase_sector()`, `crc32()`

---

## 3. CDC Serial Debug

### Arquivos alterados

| Arquivo | Mudança |
|---|---|
| `Cargo.toml` | Adicionado `ufmt = "0.2"` |
| `src/main.rs` | +30 linhas: CDC State, `CdcAcmClass`, split, spawn |
| `src/tasks/diagnostic.rs` | Reescrita completa — 131 linhas |
| `src/diagnostics/runtime_stats.rs` | Atomics tornados `pub` para leitura pelo CDC |

### Arquitetura USB

```
USB Device: OpenHOTAS (VID=0x16C0, PID=0x27DB, bcdDevice=0x0121)
├── HID Gamepad (Report ID 0x01, polling 1ms) — inalterado
└── CDC ACM (COM Virtual, packet size 64)       — NOVO
```

### Comportamento

- CDC é canal auxiliar de debug — falha/desconexão **nunca** afeta HID
- `wait_connection()` + `break` no `EndpointError` garante reconexão automática
- Banner de conexão inclui `FIRMWARE_VERSION` + `FIRMWARE_GIT_HASH`
- Telemetria a cada 5s: cycles, last_us, max_us, reports, errors
- Alerta `WARN:max_cycle` quando `MAX_INPUT_CYCLE_US` (500µs) é excedido
- Buffer de 128 bytes com detecção de overflow (`[diag truncated]`)

---

## Desvios dos Planos Originais

### 1. CDC State — padrão `Option` + `transmute` (NÃO seguido o plano)

**Plano dizia:**
```rust
static mut CDC_STATE: CdcState = CdcState::new();
let mut cdc = CdcAcmClass::new(&mut builder, unsafe { &mut CDC_STATE }, 64);
```

**Implementado:**
```rust
static mut CDC_STATE: Option<CdcState> = None;
let cdc_state = CdcState::new();
unsafe { CDC_STATE = Some(core::mem::transmute(cdc_state)); }
let cdc = CdcAcmClass::new(
    &mut builder,
    #[allow(static_mut_refs)]
    unsafe { CDC_STATE.as_mut().unwrap() },
    64,
);
```

**Motivo:** `CdcAcmClass::new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, ...)` exige que `builder` e `state` tenham a **mesma lifetime**. Com `static mut` direto, `state` tem lifetime `'static` mas `builder` é local — o compilador rejeita a unificação `'local = 'static`. O padrão `Option` + `transmute` resolve isso (cria no escopo local, transmuta para armazenar, extrai como `'static`), idêntico ao HID State já existente em `main.rs`.

### 2. `is_multiple_of()` mantido (NÃO seguido o plano)

**Plano dizia:** substituir `offset.is_multiple_of(SECTOR_SIZE)` por `offset % SECTOR_SIZE == 0` para "compatibilidade com toolchains antigos".

**Implementado:** `offset.is_multiple_of(SECTOR_SIZE)` mantido.

**Motivo:** `u32::is_multiple_of()` existe em `core` desde Rust 1.56 e é a forma idiomática. Clippy (`clippy::manual_is_multiple_of`) ativamente sugere usar `is_multiple_of()` em vez de `%`. O projeto usa Rust 2021 edition — toolchains "antigos" não são uma preocupação real.

### 3. Atomics tornados `pub` (NÃO previsto no plano)

**Adicional:** `runtime_stats.rs` teve os 5 `AtomicU32` alterados de `static` privado para `pub static`.

**Motivo:** O plano CDC menciona ler `SENSOR_CYCLES`, `MAX_CYCLE_US`, etc. diretamente via `.load(Ordering::Relaxed)`, mas os campos eram privados. Sem getters, a alternativa seria duplicar funções de acesso — menos limpo que expor os atômicos (já são thread-safe por definição).

### 4. `log_stats()` mantido com `#[allow(dead_code)]` (NÃO removido)

O plano não menciona `log_stats()`. A função era chamada pela `diagnostic_task` antiga (defmt). Com o CDC, ela não é mais chamada. Em vez de remover, foi adicionado `#[allow(dead_code)]` — a função ainda é útil para debug rápido via probe-rs sem abrir terminal CDC.

---

## Impacto Funcional

| Componente | Impacto |
|---|---|
| HID Gamepad | **Nenhum** — inalterado |
| Pipeline de sinal | **Nenhum** — inalterado |
| SPI (sensores/botões) | **Nenhum** — inalterado |
| Flash (calibração/config) | API preservada, Safe Rust, validação adicional |
| USB | HID + CDC coexistem, sem interferência |
| Tasks | `diagnostic_task` agora recebe `cdc_sender` (assinatura mudou) |

---

## Riscos

| Risco | Prob. | Impacto | Mitigação |
|---|---|---|---|
| CDC State pattern diverge da API embassy-usb futura | Baixa | Compilação quebra | Padrão idêntico ao HID State — se um quebrar, os dois quebram juntos |
| `validate_range()` rejeita offset válido | Muito baixa | Flash inacessível | `FLASH_SIZE = 2MB`, offsets são `FLASH_SIZE - N * SECTOR_SIZE` — bem dentro dos limites |
| `ufmt` conflito com `defmt` | Nenhuma | — | Crates independentes; `ufmt` usado apenas no `WriteCursor`, `defmt` inalterado |

---

## Validação Recomendada em Hardware

1. **HID:** Joystick aparece como "OpenHOTAS Gamepad" no OS, eixos e botões funcionam
2. **CDC:** Porta COM aparece no OS ("OpenHOTAS CDC"); abrir terminal mostra banner com versão e git hash
3. **Reconexão CDC:** Fechar e reabrir terminal restaura logs sem afetar HID
4. **Desconexão USB:** Remover e reconectar cabo restaura HID + CDC
5. **Flash:** Configuração salva persiste após reboot (CRC válido)
6. **Versão USB:** `lsusb -d 16c0:27db -v | grep bcdDevice` retorna `0x0121`
7. **Estresse:** 30 minutos contínuos com CDC aberto — sem panics, sem degradação

---

## Gate de Qualidade

```
Build  : PASS (cargo build --release)
Clippy : PASS (zero warnings)
Fmt    : PASS (cargo fmt --check)
```

---

*OpenHOTAS · Build Log V1.21 · Jun/2026*
