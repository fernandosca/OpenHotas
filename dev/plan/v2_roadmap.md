# OpenHOTAS — Roadmap V2

> Documento de visão. Não é um plano de implementação detalhado.
> Cada item aqui vira um arquivo próprio em `plan/` quando chegar a hora.

---

## Posição no Roadmap Geral

```
V1.0   — Build inicial (compilou)
V1.1   — Correções MT6826S + tasks/
V1.2   — Refatoração safe Rust
V1.21  — CDC Serial debug + versionamento
V2.0   — Features abaixo  ← este documento
```

---

## Features Previstas para V2

### 1. Calibração em Runtime

O esqueleto já existe com `#[allow(dead_code)]`. Precisa de:

- `Calibration::start_calibration()` / `feed()` / `finish_calibration()` — já implementados
- `cal_store::save()` — já implementado
- Interface de ativação: comando via USB HID (Report ID `0x02`, reservado)
- Flow: usuário envia comando → firmware inicia captura → usuário move eixo → firmware salva

**Arquivos existentes relevantes:**
- `calibration/data.rs` — `Calibration` struct com métodos stub
- `calibration/cal_store.rs` — `save()` com `#[allow(dead_code)]`

### 2. Configurador PC (USB HID Bidirecional)

- Report ID `0x02` já reservado em `descriptor.rs`
- Nova `config_task` para receber comandos
- Novo `usb/hid_config.rs` — handler de reports de configuração
- PC envia: novo `AxisConfig` → firmware salva em flash e aplica em runtime
- `AxisPipeline::update_config()` já existe com `#[allow(dead_code)]`

### 3. Múltiplos Perfis

- `active_profile: u8` já existe em `DeviceConfig` (sempre 0 na V1.x)
- `config/profiles.rs` — placeholder previsto
- Máximo: 4 perfis (limitado pelo espaço de flash disponível)
- Seleção: via botão físico ou comando USB

### 4. ResponseCurve Customizável

- `filters/response_curve.rs` é pass-through na V1.x
- V2: tabela lookup com pontos configuráveis pelo usuário
- Interface: enviada pelo configurador PC

### 5. Diagnóstico Expandido

O CDC adicionado na V1.21 é a base. Expansões previstas:

- Status individual de cada encoder (X, Y, Twist OK/ERR)
- Versão do firmware no header do output
- Contadores de erros por tipo (CRC, magneto, SPI)

**Nota:** `SensorStatus` em `diagnostics/sensor_health.rs` já tem a struct,
mas não está sendo alimentada pela `input_task` na V1.x.

---

## O que NÃO está no escopo do V2

- **Throttle** — projeto independente em outro Pico 2, para sempre fora deste firmware
- Eixos além dos 3 existentes (X, Y, Twist)
- Bluetooth ou Wi-Fi
- Display ou LEDs (possível V3)
- CAN Bus (possível V3)

---

## Stubs Existentes (dead_code intencional)

| Módulo | Item | Para qual feature V2 |
|---|---|---|
| `calibration/data.rs` | `Calibration::start/finish/feed` | Calibração em runtime |
| `calibration/cal_store.rs` | `save()` | Calibração em runtime |
| `config/settings.rs` | `DeviceConfig::save()` | Configurador PC |
| `config/settings.rs` | `active_profile` | Múltiplos perfis |
| `filters/ema.rs` | `set_alpha()` | Configurador PC |
| `filters/max_jump.rs` | `set_threshold()` | Configurador PC |
| `filters/deadzone.rs` | `set_threshold()` | Configurador PC |
| `filters/expo.rs` | `set_factor()` | Configurador PC |
| `filters/response_curve.rs` | interface completa | ResponseCurve customizável |
| `axis/pipeline.rs` | `update_config()` | Configurador PC |
| `usb/descriptor.rs` | `REPORT_ID_CONFIG = 0x02` | Configurador PC |
| `diagnostics/sensor_health.rs` | `SensorStatus` completo | Diagnóstico expandido |

---

## Regras que Continuam Valendo em V2

- `input_task` permanece monolítica — não dividir em V2
- Escopo: apenas joystick — throttle é sempre projeto separado
- Naming contract — prefixos `MAGIC_`, `CONFIG_`, `CALIB_` obrigatórios
- Flash: erase antes de write, nunca em ISR
- Pipeline: ordem absoluta (calibração → MaxJump → EMA → Deadzone → Expo → ResponseCurve)

---

*OpenHOTAS · Roadmap V2 · Jun/2026*
