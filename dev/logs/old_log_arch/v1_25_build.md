# OpenHOTAS — V1.25 Build Log

**Data:** 18-21/Jun/2026
**Base:** V1.24
**Tipo:** Feature release + correções + limpeza

---

## Resumo

V1.25 é uma versão significativa que:
1. Implementa Response Curve (piecewise linear 5 pontos)
2. Remove o filtro Expo
3. Corrige 6 bugs de firmware
4. Alinha GUI ↔ Firmware ↔ Protocolo
5. Atualiza contratos em dev/context/
6. Limpa dead code

---

## Features Implementadas

| Feature | Status |
|---------|--------|
| Response Curve piecewise linear | ✅ |
| Remoção do filtro Expo | ✅ |
| Correções de bugs (6 itens) | ✅ |
| Alinhamento GUI/Firmware/Protocol | ✅ |
| Atualização de contratos | ✅ |
| Limpeza de dead code | ✅ |

---

## 1. Response Curve — Piecewise Linear

### Mudança

Substituído `expo.rs` (curva cúbica simples) por `response_curve.rs`
(curva piecewise linear com 5 pontos de controle).

### Design

- 5 pontos: P0=(-1,-1), P1, P2=(0,0), P3, P4=(1,1)
- P0, P2, P4 fixos (endpoints + centro)
- P1 e P3 variáveis (controlados pelo usuário)
- Interpolação linear entre pontos adjacentes

### Justificativa

- Mais flexível que expo (curvas assimétricas, S-curves)
- Mais intuitivo (presets no configurador)
- Subsume completamente a funcionalidade do expo

### Pipeline Atualizado

```
cal → travel → maxjump → ema → deadzone → response
```

### Arquivos

| Arquivo | Mudança |
|---------|---------|
| `filters/response_curve.rs` | Reescrito: piecewise linear |
| `filters/expo.rs` | **Removido** |
| `axis/mod.rs` | `expo` → `response_p1`, `response_p3` |
| `axis/pipeline.rs` | Expo removido, response integrado |
| `config/runtime.rs` | Conversão `ResponseCurveData` → `(f32, f32)` |
| `constants.rs` | Removido `DEFAULT_EXPO` |
| `openhotas-protocol/src/config.rs` | `ResponseCurveData`, presets |

---

## 2. Correções de Bugs

| # | Bug | Arquivo | Correção |
|---|-----|---------|----------|
| 1 | Protocol version mismatch | `protocol.ts` | `1` → `2` |
| 2 | CRC8 overflow debug | `mt6826.rs` | `wrapping_shl(1)` |
| 3 | Deadzone div/zero | `deadzone.rs` | Guard `threshold >= 1.0` |
| 4 | SPI bus expect() | `spi_bus.rs` | `Result<R, SpiBusError>` |
| 5 | read_flash init check | `flash.rs` | Verificação antes de XIP |
| 6 | GIT_HASH sem fallback | `constants.rs` | `option_env!` + `"unknown"` |

---

## 3. Alinhamento GUI/Firmware/Protocol

### Protocol Crate
- `CurvePoint`, `ResponseCurveData` adicionados
- `expo_i16` removido de `AxisConfig`
- `PROTOCOL_VERSION_MAJOR` 1 → 2

### GUI
- `protocol.ts`: Tipos atualizados
- `CurveEditor.tsx`: Piecewise linear com pontos P1/P3
- `CurvePage.tsx`: Presets (Linear, Suave, Centro, S-curve)

### CLI
- `commands.rs`: `curve_preset` em vez de `expo_pct`
- `display.rs`: `response_curve` em vez de `expo_i16`

---

## 4. Limpeza de Dead Code

| Item | Arquivo | Ação |
|------|---------|------|
| `REPORT_ID_CONFIG` | `constants.rs` | Removido |
| `#[allow(dead_code)]` | `axis/mod.rs` | Removido de `healthy` |

---

## 5. Atualização de Contratos

| Arquivo | Mudança |
|---------|---------|
| `01_architecture.md` | Expo removido, response_curve atualizado |
| `04_software_contracts.md` | Pipeline 6 etapas, justificativa |
| `05_coding_guidelines.md` | Pipeline order, stubs V2 |
| `dev/CLAUDE.md` | Pipeline e stubs V2 |

---

## Gate de Qualidade

```
Build     : PASS
Clippy    : PASS
Fmt       : PASS
TypeScript: PASS
```

---

*OpenHOTAS · V1.25 Build Log · Jun/2026*
