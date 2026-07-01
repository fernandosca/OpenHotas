# Análise Completa do Firmware OpenHOTAS (v2 — Ajustada)

**Data original:** 1 Jul 2026
**Ajustes:** repriorização + notas de implementação
**Branch:** `codex/v1.3-fixes` (commit `b2233a4`)
**Revisores:** 3 subagentes (arquitetura, sensores, USB/tasks)

---

## Status dos Issues

| Issue | Status | Data |
|-------|--------|------|
| I-2 | ✅ Implementado | 1/Jul/2026 |
| I-5 | ✅ Implementado | 1/Jul/2026 |
| I-4 | ✅ Implementado | 1/Jul/2026 |
| I-1 | ⏳ Pendente | — |
| I-3 | ✅ Implementado | 1/Jul/2026 |
| M-1 | ✅ Implementado | 1/Jul/2026 |
| M-2 | ✅ Implementado | 1/Jul/2026 |
| M-3 | — | Aceitável (diagnóstico) |
| M-4 | ✅ Implementado | 1/Jul/2026 |
| M-5 | ✅ Implementado | 1/Jul/2026 |

---

## Issues Pendentes

### I-1. `static mut` deprecated no Rust 2024

- **Arquivo:** `main.rs:41-48`
- **Problema:** 7 variáveis `static mut` (`DD`, `CD`, `BD`, `CB`, `HS`, `CDC_STATE`, `SERIAL_STR`) são deprecated na edição Rust 2024
- **Impacto:** Vai quebrar na migração de edição
- **Status:** Bloqueado — requer adição de crate `static-cell` e refatoração significativa

**Nota de implementação:** Não é urgente pra funcionalidade, mas vale resolver antes de o diagnostic mode entrar (que provavelmente vai querer mais um `AtomicBool` compartilhado — melhor não criar mais um `static mut` pra depois migrar junto). Migre para `StaticCell`:

```rust
static DD: StaticCell<DeviceData> = StaticCell::new();
let dd: &'static mut DeviceData = DD.init(DeviceData::default());
```

Faça isso variável por variável, não tudo de uma vez — cada `static mut` tem um padrão de acesso ligeiramente diferente (algumas são lidas só no `input_task`, outras compartilhadas entre tasks), e migrar em lote aumenta a chance de introduzir uma race que não existia antes.

---

## Issues Implementados (Referência)

### I-2. `save_config` — Double-Buffer ✅

- **Arquivo:** `config/stored_config_v2.rs`
- **Solução:** Dois setores flash com geração + CRC32

### I-5. `build_frame` buffer — Compile-Time Assertion ✅

- **Arquivo:** `tasks/cdc.rs`
- **Solução:** `const _: () = assert!(4 + MAX_PAYLOAD_SIZE + 2 <= 300, "...");`

### I-4. Calibração — Snapshot Atômico ✅

- **Arquivo:** `tasks/cdc_handlers.rs`
- **Solução:** `critical_section::with(|_| { ... })` envolvendo leituras

### I-3. `MAX_CYCLE_US` — Reset em Janela Deslizante ✅

- **Arquivo:** `diagnostics/runtime_stats.rs`
- **Solução:** Função `reset_max_cycle()` chamada pelo diagnostic_task

### M-1. Constante Morta `REPORT_ID_GAMEPAD` ✅

- **Arquivo:** `constants.rs`
- **Solução:** Removida

### M-2. `axis_to_i16` — Safety Comment ✅

- **Arquivo:** `usb/hid_gamepad.rs`
- **Solução:** Comentário documentando a invariante

### M-4. `as_micros()` — Truncation Protection ✅

- **Arquivo:** `tasks/input.rs`
- **Solução:** `.min(u64::from(u32::MAX))`

### M-5. Magic Number `0xA0` ✅

- **Arquivo:** `sensors/mt6826.rs`
- **Solução:** `MT6826_CMD_READ_ANGLE << 4`

---

## Pontos Fortes Validados

- **Pipeline correto** — Cal → CenterOffset → Travel → MaxJump → EMA → Deadzone → Response
- **Clamp defense-in-depth** — Todo filtro clampa `[-1.0, 1.0]` em entrada E saída
- **SPI bus sharing** — Padrão closure+Mutex previne lock leaks
- **Flash XIP TOCTOU fix** — Lock mantido durante leitura XIP
- **Zero heap allocation** — Design `no_std` limpo
- **Sensor absent detection** — 0xFF no MISO com pull-up
- **MCP23S17 init verification** — Read-back confere presença do chip
- **USB serial from OTP** — Chip ID único por placa

---

## Veredicto

**8 de 9 issues implementados.** O único pendente (I-1) é bloqueado pela necessidade de adicionar crate `static-cell`. O firmware está em bom estado para uso, com power-fail safety implementado via double-buffer.
