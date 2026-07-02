# OpenHOTAS — V1.26 Build Log

**Data:** 19/Jun/2026
**Base:** V1.25
**Tipo:** GUI + CLI + Estabilidade

---

## Resumo

V1.26 foca em três áreas:
1. **GUI:** Reorganização visual do configurador
2. **CLI:** Endurecimento de contrato e auto-detect
3. **Firmware:** Estabilidade e robustez

---

## 1. GUI — Changelog Visual

### Dashboard convertido em tela de eixos

- `Dashboard` consolidado como tela `Eixos`
- Grid de botões movido para tela separada
- Canvas HUD ajustado para Twist/Z

### Nova tela de botões

- Card de configuração movido para página `Botões`
- Grid de 32 botões + lista ativa + masks + debounce

### Tela de curvas

- Gráfico simplificado para visualização (sem edição de pontos)
- Presets: Linear, Suave, Centro, S-curve
- Slider de deadzone mantido

### Calibração consolidada

- Fluxo em único card
- Coluna lateral removida
- Status e instruções centralizados

### Diagnósticos

- Card de sensores unificado com Runtime stats
- Botão "Atualizar tudo" integrado

---

## 2. CLI — Contract Hardening

### Handshake obrigatório

`OpenHotasTransport::connect_to()` envia `GetInfo` e valida:
- `protocol_major == PROTOCOL_VERSION_MAJOR`
- `axis_count == 3`
- `button_count == 32`

### Auto-detect por identidade

`connect()` não aceita mais a primeira `/dev/ttyACM*`.
Agora tenta cada porta e valida handshake `GetInfo`.

---

## 3. Firmware — Estabilidade

### input_task cadenciada por Ticker

```rust
Ticker::every(Duration::from_micros(500))
```

Preserva meta de 500µs e permite escalonamento das outras tasks.

### Calibração — sensor unhealthy

`CaptureCalibrationPoint` rejeita captura se eixo está unhealthy.
Previte persistir fallback de centro como leitura válida.

### Sessão CDC — cleanup

Sessão de calibração é limpa ao desconectar CDC.
Previse estado órfão se PC desconectar durante calibração.

### Configurador — polling suspendido

Polling do configurador é pausado durante operações de
escrita/configuração para evitar conflitos.

---

## Arquivos Alterados (Resumo)

### GUI
- `App.tsx`: Navegação reorganizada
- `Dashboard.tsx`: Consolidado como Eixos
- `ButtonsPage.tsx`: Nova tela de botões
- `CurvePage.tsx`: Presets simplificados
- `Calibration.tsx`: Layout consolidado
- `Diagnostics.tsx`: Cards unificados

### CLI
- `transport.rs`: Handshake `GetInfo`, auto-detect

### Firmware
- `tasks/input.rs`: `Ticker::every(500µs)`
- `tasks/cdc_handlers.rs`: Validação de saúde na calibração
- `tasks/cdc.rs`: Cleanup de sessão

---

## Gate de Qualidade

```
Firmware   : PASS
CLI        : PASS
GUI        : PASS (TypeScript)
```

---

*OpenHOTAS · V1.26 Build Log · Jun/2026*
