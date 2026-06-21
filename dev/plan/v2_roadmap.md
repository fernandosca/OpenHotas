# OpenHOTAS — Roadmap V2

> Documento de visão. Apenas features NÃO implementadas.
> Features implementadas estão documentadas em `dev/logs/`.

---

## Posição no Roadmap Geral

```
V1.0   — Build inicial (compilou)
V1.1   — Correções MT6826S + tasks/
V1.2   — Refatoração safe Rust
V1.21  — CDC Serial debug + versionamento
V1.23  — StoredConfigV2, calibração via CDC, diagnóstico expandido
V1.25  — ResponseCurve piecewise linear, correções de bugs, limpeza
V1.3   — Axis-to-button, Center offset, MCP23S17 burst read
V2.0   — Features abaixo  ← este documento
```

---

## Features Previstas para V2 (Não Implementadas)

### 1. Múltiplos Perfis

- Seleção de perfis de configuração (ex: "Combate", "Simulação", "Cruzeiro")
- Máximo: 4 perfis (limitado pelo espaço de flash)
- Seleção: via botão físico ou comando USB
- Cada perfil armazena: calibração, filtros, limites de curso, botões

**Status:** Removido em V1.23. Possível reconsideração em V3 se houver demanda.

### 2. HID Config Protocol (Report ID 0x02)

- Configuração via HID em vez de CDC
- Report ID `0x02` reservado no descriptor
- Vantagem: não precisa de porta serial separada
- Desvantagem: CDC já funciona, complexidade adicional sem benefício claro

**Status:** CDC utilizado. HID config é alternativa não priorizada.

### 3. Button Long Press

- Detectar press longo (>500ms) como botão virtual separado
- Dobra o número de funções disponíveis
- Complexidade: ⭐⭐

### 4. Button Toggle

- Modo toggle para botões (liga/desliga)
- Útil para gear, flaps, lights
- Complexidade: ⭐⭐

### 5. Sensitivity Per Axis

- Multiplicador de sensibilidade por eixo (50-200%)
- Ajuste fino sem mudar travel limits
- Complexidade: ⭐

---

## O que NÃO está no escopo

- **Throttle** — projeto independente em outro Pico 2
- Eixos além dos 3 existentes (X, Y, Twist)
- Bluetooth ou Wi-Fi
- Display ou LEDs (possível V3)
- CAN Bus (possível V3)

---

## Regras que Continuam Valendo

- `input_task` permanece monolítica — não dividir
- Escopo: apenas joystick — throttle é sempre projeto separado
- Flash: erase antes de write, nunca em ISR
- Pipeline: calibração → center_offset → Travel → MaxJump → EMA → Deadzone → ResponseCurve

---

## Implementações V1.x (Documentadas em dev/logs/)

| Versão | Feature | Log |
|--------|---------|-----|
| V1.23 | Calibração via CDC | `v1_23_build.md` |
| V1.23 | Configurador PC (CDC) | `v1_23_build.md` |
| V1.23 | Diagnóstico expandido | `v1_23_build.md` |
| V1.25 | ResponseCurve piecewise linear | `v1_25_build.md` |
| V1.25 | Correções de bugs (6 itens) | `v1_25_build.md` |
| V1.25 | Limpeza de dead code | `v1_25_build.md` |
| V1.3 | Axis-to-button | `v1_3_build.md` |
| V1.3 | Center offset | `v1_3_build.md` |
| V1.3 | MCP23S17 burst read | `v1_3_build.md` |

---

*OpenHOTAS · Roadmap V2 · V1.3.0 · Jun/2026*
