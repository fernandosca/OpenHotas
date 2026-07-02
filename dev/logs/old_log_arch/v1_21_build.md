# OpenHOTAS — V1.21 Build Log

**Data:** 17-18/Jun/2026
**Versão:** 1.2.1
**Gate:** ✅ Build ✅ Clippy ✅ Fmt

---

## Resumo

V1.21 implementa 3 features planejadas:

| # | Feature | Status |
|---|---|---|
| 1 | Sistema de Versionamento | ✅ |
| 2 | Flash Driver Seguro (Safe Rust) | ✅ |
| 3 | CDC Serial Debug | ✅ |

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

- **Três camadas de versionamento:** Cargo.toml (SemVer → logs), build.rs (git hash → rastreabilidade), USB descriptor (BCD → visível no host)
- **`device_release` manual:** BCD requer formato `0xMMmm`; conversão de string não é trivial em `no_std`
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
| `static mut FLASH_INSTANCE` | `Mutex<CriticalSectionRawMutex, RefCell<Option<...>>>` |
| `critical_section::with` manual | `embassy_sync::Mutex` |
| `FlashError` com 4 variantes | + `OutOfBounds`, `NotInitialized` |
| Sem validação de limites | `validate_range()` com `checked_add` |

### Justificativas

- **Padrão `Mutex<...>`:** Idêntico ao `spi_bus.rs` (V1.2). Consistência arquitetural
- **`NotInitialized` vs `WriteError`:** Diagnóstico mais preciso
- **`validate_range()`:** Previne acesso fora dos 2MB da flash
- **Único `unsafe` restante:** `core::ptr::read_volatile` para leitura XIP

---

## 3. CDC Serial Debug

### Arquivos alterados

| Arquivo | Mudança |
|---|---|
| `Cargo.toml` | Adicionado `ufmt = "0.2"` |
| `src/main.rs` | +30 linhas: CDC State, `CdcAcmClass`, split, spawn |
| `src/tasks/diagnostic.rs` | Reescrita completa — 131 linhas |
| `src/diagnostics/runtime_stats.rs` | Atomics tornados `pub` |

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

---

## Desvios dos Planos Originais

| Item | Plano | Implementado | Motivo |
|---|---|---|---|
| CDC State | `static mut` direto | `Option` + `transmute` | Lifetime mismatch builder/state |
| `is_multiple_of` | Substituir por `%` | Mantido `is_multiple_of()` | Forma idiomática desde Rust 1.56 |
| Atomics `pub` | Não previsto | Expostos como `pub static` | CDC precisa ler via `.load()` |
| `log_stats()` | Não mencionado | `#[allow(dead_code)]` | Útil para debug via probe-rs |

---

## Validação em Hardware

1. **HID:** Joystick aparece como "OpenHOTAS Gamepad"
2. **CDC:** Porta COM aparece, terminal mostra banner
3. **Reconexão CDC:** Fechar/reabrir terminal restaura logs
4. **Flash:** Config salva persiste após reboot
5. **Versão USB:** `lsusb` retorna `bcdDevice=0x0121`

---

*OpenHOTAS · V1.21 Build Log · Jun/2026*
