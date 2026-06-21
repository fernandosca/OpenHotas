# OpenHOTAS — V1.3.0 Build Log

**Data:** 21/Jun/2026
**Base:** V1.25
**Tipo:** Feature release — novas funcionalidades

---

## Resumo

V1.3.0 adiciona três funcionalidades significativas:

| # | Feature | Complexidade | Impacto |
|---|---------|-------------|---------|
| 1 | Axis-to-Button | Média | Dobra funcionalidade dos eixos |
| 2 | Center Offset | Baixa | Ajuste fino sem recalibrar |
| 3 | MCP23S17 Burst Read | Baixa | 50% menos transações SPI |

---

## 1. Axis-to-Button

### Feature

Mapeia posição de eixo para botão virtual no report HID.
O eixo continua funcionando normalmente, mas simultaneamente
ativa um bit no campo de botões quando atinge um limiar.

### Casos de uso (Flight Simulator)

- **Speed Brake:** Twist > 80% ativa "Speed Brake OPEN"
- **Flaps:** Y > 60% ativa "Flaps Down"
- **Trim:** Twist > 90% ativa "Trim Up"
- **Gear:** Y < -80% ativa "Gear Down"

### Configuração

```rust
pub struct AxisToButtonConfig {
    pub enabled: bool,           // Default: false
    pub threshold_permille: u16, // 0..1000 (800 = 80%)
    pub direction: AxisDirection, // Positive, Negative, Both
    pub button_index: u8,        // 0..31
}
```

### Pipeline

```
Eixo (f32 -1.0 a +1.0)
        │
        ▼
┌─────────────────┐
│ axis_to_button  │
│ threshold: 0.8  │
│ direction: Both │
│ button: 28      │
└────────┬────────┘
         │
         ├──→ Eixo original (normal)
         └──→ Botão virtual (bit 28 no report)
```

### Validação de Colisão

O firmware valida que:
1. Botões virtuais não colidem entre si

Botões virtuais podem compartilhar o mesmo índice de um botão físico.
O report final compõe as fontes por OR: botão físico pressionado **ou**
eixo acima do limiar ativam o mesmo bit HID. Isso evita desabilitar o botão
físico quando ele também é usado como destino virtual.

### Arquivos

| Arquivo | Mudança |
|---------|---------|
| `openhotas-protocol/src/config.rs` | `AxisDirection`, `AxisToButtonConfig` |
| `firmware/src/config/runtime.rs` | `AxisToButtonRuntime`, conversão, validação |
| `firmware/src/tasks/input.rs` | `apply_axis_to_button()` + chamada no loop |

---

## 2. Center Offset

### Feature

Ajusta o ponto zero do eixo via offset em permille.
Útil para corrigir desalinhamento mecânico sem recalibrar.

### Exemplo Prático

```
Problema: Magneto montado 1mm para a direita
          Centro mecânico = 0.0, mas eixo lê +2%

Sem Center Offset:
  - Usuário segura no centro → eixo mostra +2%

Com Center Offset = -20 permille:
  - Firmware aplica: output = valor + (-0.02)
  - Centro mecânico → output = 0.0 ✓
```

### Configuração

```rust
pub center_offset_permille: i16,  // -200..200 (±20%)
```

### Pipeline Atualizado

```
cal → center_offset → travel → maxjump → ema → deadzone → response
```

### Arquivos

| Arquivo | Mudança |
|---------|---------|
| `openhotas-protocol/src/config.rs` | `center_offset_permille` em `AxisConfig` |
| `firmware/src/config/runtime.rs` | `center_offset: f32`, conversão |
| `firmware/src/axis/pipeline.rs` | Campo + aplicação após calibração |

---

## 3. MCP23S17 Burst Read

### Mudança

Substituída leitura sequencial (2 transações SPI por chip) por burst read
(1 transação SPI por chip).

### Antes vs Depois

| Métrica | Antes | Depois |
|---------|-------|--------|
| Transações SPI | 4 | 2 |
| Overhead CS | 4× | 2× |
| Atomicidade | GPIOA/B separados | GPIOA/B juntos |

### Como funciona

MCP23S17 suporta leitura sequencial com auto-incremento.
Lendo a partir de 0x12 (GPIOA), retorna GPIOA + GPIOB em uma transação.

### Arquivos

| Arquivo | Mudança |
|---------|---------|
| `firmware/src/sensors/mcp23s.rs` | `read_chip_raw` → burst read |

---

## 4. Outras Mudanças

### Protocol Version

- `PROTOCOL_VERSION_MAJOR`: 2 → 3

### GUI Tipos

- `gui/src/types/protocol.ts`: `AxisToButtonConfig`, `center_offset_permille`

### GUI Controles

- `gui/src/components/dashboard/Dashboard.tsx`: controles visuais para
  `center_offset_permille` e `axis_to_button`
- `max_jump_raw` permanece no protocolo e no firmware, mas foi ocultado da GUI
  principal por ser ajuste técnico de estabilidade/anti-spike; o valor padrão
  ou carregado do dispositivo é preservado
- `axis_to_button` não altera mais `buttons.enabled_mask`; botão físico e
  botão virtual podem ativar o mesmo bit HID por composição OR

### CLI

- `cli/src/commands.rs`: `--center-offset`, `--axis-to-button`
- `cli/src/display.rs`: Exibe novos campos

### Contratos Atualizados

- `01_architecture.md`: V1.3.0, burst read, módulos
- `04_software_contracts.md`: Pipeline com center offset (7 etapas)
- `05_coding_guidelines.md`: Pipeline order
- `dev/CLAUDE.md`: Pipeline

---

## Gate de Qualidade

```
Firmware clippy (thumbv8m.main-none-eabihf): PASS
Protocol clippy                         : PASS
CLI clippy                              : PASS
GUI build / TypeScript                  : PASS
Tauri backend check                     : PASS
Fmt                                     : PASS
```

### CI / Release

- CI cobre `protocol`, `firmware`, `cli` e `gui`
- Release por tag `v*` gera firmware `.elf`/`.uf2`, CLI Linux, CLI Windows
  e artefatos GUI Windows via Tauri

---

*OpenHOTAS · V1.3.0 Build Log · Jun/2026*
