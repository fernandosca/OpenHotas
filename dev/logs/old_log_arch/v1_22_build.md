# OpenHOTAS — V1.22 Build Log

**Data:** 18/Jun/2026
**Versão:** 1.2.2
**Gate:** ✅ Build ✅ Clippy ✅ Fmt ✅ Tests (8/8)

---

## Resumo

V1.22 implementa o **Configuration Protocol** — configuração e calibração via USB CDC
com protocolo binário request/response. Esta é a versão base para todas as features
de configuração futuras.

---

## Features Implementadas

| Fase | Feature | Status |
|------|---------|--------|
| 0 | Correções HID + Workspace | ✅ |
| 1+2 | Crate `openhotas-protocol` + testes | ✅ |
| 3 | RuntimeConfig + Signal | ✅ |
| 4 | `cdc_task` read-only | ✅ |
| 5 | SetConfig runtime | ✅ |
| 6 | Flash StoredConfigV2 | ✅ |
| 7 | Calibração via CDC | ✅ |
| 8 | CLI PC | ⏳ (reservada para V1.23) |

---

## 1. Fase 0 — Correções Base

### 1.1 Corrigir HID Report ID
**Arquivo:** `firmware/src/usb/descriptor.rs`
- Removido Report ID desnecessário (apenas 1 report HID)
- Mantido `REPORT_SIZE = 10` (6 bytes eixos + 4 bytes botões)

### 1.2 Corrigir Logical Minimum dos eixos
**Arquivo:** `firmware/src/usb/descriptor.rs`
- Alterado de `+1` para `-32767` (i16 assinado)

### 1.3 Desacoplar `diagnostic_task` do CDC
**Arquivos:** `firmware/src/tasks/diagnostic.rs`, `firmware/src/main.rs`
- `diagnostic_task` voltou a logar apenas via `defmt`

---

## 2. Fase 1+2 — Crate `openhotas-protocol`

### Arquitetura

```
crates/openhotas-protocol/
├── src/
│   ├── lib.rs          # no_std, serde + postcard
│   ├── config.rs       # DeviceConfig, AxisConfig, ButtonConfig
│   ├── request.rs      # Request enum (GetInfo, GetConfig, SetConfig, ...)
│   ├── response.rs     # Response enum (Ack, Error, Config, ...)
│   ├── diagnostics.rs  # RawAxes, ProcessedAxes, RuntimeStats, ...
│   ├── error.rs        # ProtocolError enum
│   ├── frame.rs        # Frame format: SOF + LEN + PAYLOAD + CRC16
│   └── version.rs      # PROTOCOL_VERSION_MAJOR/MINOR
```

### Frame Format

```
Offset  Size  Field
0       2     SOF = 0xAA 0x55
2       2     LEN u16 big-endian
4       LEN   PAYLOAD (postcard-serialized)
4+LEN   2     CRC16-CCITT big-endian (covers LEN + PAYLOAD)
```

---

## 3. Fase 3 — RuntimeConfig + Signal

### Arquitetura de Comunicação

```
cdc_task ─── CONFIG_SIGNAL ───→ input_task
              (Channel, capacity=1, latest-wins)
```

### Decisão D-07: Channel, não Mutex

- `Channel<Capacity=1>` com `try_receive()` — input_task nunca bloqueia
- Se channel cheio, `cdc_task` drena e reenvia (latest-wins)

---

## 4. Fase 4 — cdc_task Read-only

### Handlers Implementados

| Request | Handler | Descrição |
|---------|---------|-----------|
| `GetInfo` | `handle_read_request` | Versão, hardware info |
| `GetConfig` | `handle_read_request` | Configuração ativa |
| `GetRawAxes` | `handle_read_request` | Valores crus dos sensores |
| `GetProcessedAxes` | `handle_read_request` | Valores processados (i16) |
| `GetButtonStates` | `handle_read_request` | Estado dos 32 botões |
| `GetSensorStatus` | `handle_read_request` | Saúde por sensor |
| `GetRuntimeStats` | `handle_read_request` | Estatísticas de ciclo |
| `GetErrorCounters` | `handle_read_request` | Contadores de erro |

---

## 5. Fase 5 — SetConfig Runtime

### Fluxo

```
PC ──SetConfig(DeviceConfig)──→ cdc_task
                                  │
                                  ├─ from_protocol_config() → RuntimeConfig
                                  ├─ signal_latest_config() → input_task
                                  └─ Response::Ack
```

### Validação

- Protocol version check
- Range checks (deadzone, ema, max_jump, travel)
- Calibration ordering (min < center < max)

---

## 6. Fase 6 — Flash StoredConfigV2

### Layout

```
Offset  Size  Field
0       4     MAGIC = "OCFG"
4       1     STORAGE_VERSION = 2
5       1     PROTOCOL_MAJOR
6       1     PROTOCOL_MINOR
7       2     PAYLOAD_LEN u16 big-endian
9       N     PAYLOAD = postcard(DeviceConfig)
9+N     4     CRC32
```

### Invariantes

- `SaveConfig` grava flash
- `LoadDefaults` NÃO grava flash
- `SetConfig` NÃO grava flash
- `FactoryReset` grava defaults + reboot

---

## 7. Fase 7 — Calibração via CDC

### Fluxo

```
PC ──StartCalibration(axis)──→ firmware
     ──CaptureCalibrationPoint──→ (Min, Center, Max)
     ──FinishCalibration(axis)──→ Aplica calibração
     ──SaveConfig──→ Persiste em flash
```

### Regras

- Sessão exclusiva por vez (Busy se outra ativa)
- Sensores devem estar saudáveis para capturar
- Calibração aplica em runtime mas NÃO persiste automaticamente

---

## Arquivos Alterados (Resumo)

### Firmware
- `main.rs`: CDC State, CdcAcmClass, spawn
- `tasks/cdc.rs`: Protocolo binário request/response
- `tasks/cdc_handlers.rs`: Handlers de leitura/escrita/calibração
- `config/runtime.rs`: RuntimeConfig, CONFIG_SIGNAL
- `config/stored_config_v2.rs`: Persistência flash
- `usb/descriptor.rs`: Correções HID

### Protocol Crate (novo)
- `crates/openhotas-protocol/src/`: 8 arquivos

---

## Decisões de Design

| Decisão | Escolha | Justificativa |
|---------|---------|---------------|
| D-01 | CDC para configuração | HID fica exclusivo para gamepad |
| D-02 | Request/Response | Simples, previsível, sem telemetria espontânea |
| D-04 | Postcard (serde) | Compacto, no_std, type-safe |
| D-05 | CRC16 no frame | Protege LEN + PAYLOAD contra corrupção |
| D-06 | Sem f32 no protocolo | Evita NaN, Infinity, endianness issues |
| D-07 | Channel latest-wins | input_task nunca bloqueia |
| D-08 | cdc_task owns config | Evita Mutex global, simplifica concorrência |

---

## Validação em Hardware

1. **HID:** Joystick funcional (eixos + botões)
2. **CDC:** Porta COM aparece, protocolo binário funciona
3. **Config:** `SetConfig` altera runtime, `SaveConfig` persiste
4. **Calibração:** Fluxo completo Min→Center→Max funciona
5. **Reboot:** Config persiste após reboot
6. **Factory Reset:** Restaura defaults

---

*OpenHOTAS · V1.22 Build Log · Jun/2026*
