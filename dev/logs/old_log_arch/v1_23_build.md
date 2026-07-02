# OpenHOTAS — V1.23 Build Log

**Data:** 18/Jun/2026
**Base:** V1.22
**Objetivo:** Estabilização do firmware + remoção de legacy V1 + PC CLI funcional

---

## Resumo

V1.23 é uma versão de estabilização que:
1. Corrige bugs identificados na auditoria
2. Remove código legado V1
3. Implementa o CLI funcional
4. Melhora robustez do protocolo

---

## Features Implementadas

| Feature | Status |
|---------|--------|
| Latest-wins no canal de config | ✅ |
| Reboot com Ack + delay 100ms | ✅ |
| FactoryReset com reboot | ✅ |
| Remoção de legado V1 | ✅ |
| CLI funcional | ✅ |
| Correções de auditoria | ✅ |

---

## 1. Latest-wins no Canal de Config

**Problema:** `Channel::try_send()` falhava silenciosamente se a `input_task` não
tivesse consumido a config anterior. O PC recebia `Ack` mas a config nunca chegava.

**Solução:** Helper `signal_latest_config()` — se canal cheio, drena antiga e retenta.

```rust
pub fn signal_latest_config(config: RuntimeConfig) -> bool {
    match CONFIG_SIGNAL.try_send(config) {
        Ok(()) => true,
        Err(TrySendError::Full(cfg)) => {
            let _ = CONFIG_SIGNAL.try_receive();
            CONFIG_SIGNAL.try_send(cfg).is_ok()
        }
    }
}
```

---

## 2. Reboot com Ack + Delay

**Problema:** `sys_reset()` chamado antes do buffer USB ser transmitido.

**Solução:** `Timer::after_millis(100).await` antes de `SCB::sys_reset()`.

---

## 3. Remoção de Legado V1

### Arquivos removidos

| Arquivo | Motivo |
|---------|--------|
| `cal_store.rs` | Substituído por `stored_config_v2.rs` |
| `settings.rs` | Substituído por `stored_config_v2.rs` |
| `sensor_health.rs` | Substituído por `runtime_stats.rs` |

### Constantes removidas

| Constante | Motivo |
|-----------|--------|
| `CALIB_OFFSET` | Layout V1 obsoleto |
| `CONFIG_OFFSET` | Layout V1 obsoleto |
| `MAGIC_*` | Substituído por `MAGIC_V2` |

---

## 4. Auditoria de Código

### Issues Identificados e Corrigidos

| # | Issue | Prioridade | Status |
|---|-------|------------|--------|
| 1 | `CaptureCalibrationPoint` rejeita `raw == 0` | Alta | ✅ Corrigido |
| 2 | `expo_i16` negativo ignorado | Alta | ✅ Corrigido |
| 3 | `save_config` 4KB na stack | Alta | ✅ Corrigido |
| 4 | `Channel::try_send` silencioso | Alta | ✅ Corrigido |
| 5 | CRC protocolo vs CRC sensor | Alta | ✅ Separados |
| 6 | `max_jump_raw` com calibração | Média | ✅ Corrigido (V1.24) |
| 7 | Reset EMA ao reabilitar eixo | Média | ✅ Corrigido (V1.25) |

---

## 5. CLI Funcional

### Comandos Implementados

```sh
openhotas-cli info
openhotas-cli get-config
openhotas-cli set-axis --axis x --deadzone 5
openhotas-cli save
openhotas-cli calibrate --axis x
openhotas-cli raw-axes
openhotas-cli processed-axes
openhotas-cli buttons
openhotas-cli stats
openhotas-cli errors
openhotas-cli sensor-status
openhotas-cli load-defaults
openhotas-cli reboot
openhotas-cli factory-reset
```

---

## Arquivos Alterados (Resumo)

### Firmware
- `config/runtime.rs`: `signal_latest_config()`, validação melhorada
- `tasks/cdc.rs`: Delay de reboot, latest-wins
- `tasks/cdc_handlers.rs`: Correção `raw == 0`
- `config/stored_config_v2.rs`: Stack buffer otimizado

### Removidos
- `cal_store.rs`, `settings.rs`, `sensor_health.rs`

### Adicionados
- `cli/` — Ferramenta CLI completa

---

## Gate de Qualidade

```
Build  : PASS
Clippy : PASS
Fmt    : PASS
```

---

*OpenHOTAS · V1.23 Build Log · Jun/2026*
