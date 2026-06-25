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
- `openhotas-protocol`: versionado como `1.3.0` para rastrear a build V1.3

### GUI Tipos

- `gui/src/types/protocol.ts`: `AxisToButtonConfig`, `center_offset_permille`
- GUI versionada como `1.3.0` em `package.json`, `tauri.conf.json` e
  `src-tauri/Cargo.toml`

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
- CLI versionada como `1.3.0`

### Contratos Atualizados

- `01_architecture.md`: V1.3.0, burst read, módulos
- `04_software_contracts.md`: Pipeline com center offset (7 etapas)
- `05_coding_guidelines.md`: Pipeline order
- `dev/CLAUDE.md`: Pipeline

---

## 5. Code Quality — Session Improvements (Jun/2026)

### SERIAL_STR Refactor

O padrão de inicialização do serial USB foi simplificado:

| Antes | Depois |
|-------|--------|
| Raw pointer writes byte-a-byte | Buffer local + `copy_nonoverlapping` |
| `unsafe` aninhado (3 blocos) | 1 único `unsafe` block |
| `addr_of_mut!` + `ptr.write()` | Formatação safe no buffer local |

O loop de formatação agora usa operações safe com slice indexing no buffer
local, e apenas `copy_nonoverlapping` + `from_raw_parts` ficam no `unsafe`.

### Serial USB Contract (§8.1)

Adicionado em `dev/context/04_software_contracts.md`:
- Formato: `"OH{:016X}"` (18 bytes)
- Fonte: CHIP_ID registers do RP2350
- Contrato para ferramentas: serial é uniqueness, não discovery

### Doc Version Alignment

Rodapés atualizados para V1.3.0:
- `04_software_contracts.md`: V1.23 → V1.3.0
- `05_coding_guidelines.md`: V1.2 → V1.3.0

### Gate de Qualidade (pós-melhorias)

```
FMT      : PASS
Build    : PASS
Clippy   : PASS (zero warnings)
```

---

## 6. CHIP_ID Fix — OTP Unique ID (Jun/2026)

### Problema

Endereços `0x00010040`/`0x00010044` para leitura de CHIP_ID estavam **incorretos**:
- Base `0x00010000` não é SYSINFO (`0x40000000`)
- Leitura funcionava por acaso (lixo da região ROM sem crash)
- Serial USB poderia ser igual em todas as placas

### Solução

Substituído por `otp::get_chipid()` — lê chip ID de OTP (rows 0x0-0x3), retorna `u64`.
API segura do embassy-rp, sem raw pointers.

### Mudanças

| Arquivo | Mudança |
|---------|---------|
| `firmware/src/main.rs` | `chip_id_serial_static()` usa `otp::get_chipid()` em vez de endereços incorretos |
| `firmware/src/main.rs` | Removidas constantes `CHIP_ID_HI`/`CHIP_ID_LO` |
| `firmware/src/main.rs` | Adicionado `use embassy_rp::otp` |

### Gate de Qualidade

```
Build    : PASS
Clippy   : PASS (zero warnings)
FMT      : PASS
```

---

## 7. Análise de Ajustes (Jun/2026)

### A. expect() em chip_id_serial_static()

**Status:** Já implementado (sem alteração necessária).

O código em `main.rs:225` já usa `unwrap_or_else` com fallback:
```rust
let chip_id = otp::get_chipid().unwrap_or_else(|_| {
    defmt::warn!("OTP chip ID unavailable, using fallback serial");
    0u64
});
```

Em caso de falha OTP, o serial vira `OH0000000000000000` e o boot continua
funcional. Alinhado com o estilo defensivo do projeto (MCP → buttons released,
sensor → centro).

### B. Contrato do serial — 04_software_contracts.md §8.1

**Status:** Já documentado (sem alteração necessária).

A §8.1 (linhas 329-334) já menciona OTP como fonte oficial:
> **Fonte:** chip ID de OTP, lido via `embassy_rp::otp::get_chipid()` (rows 0x0–0x3).

E inclui warning contra SYSINFO:
> ⚠️ Não usar os endereços SYSINFO — base incorreta.

O log §5 (linhas 211-212) mencionava "CHIP_ID registers" — linguagem
ambígua, mas o contrato §8.1 já está preciso. Não há inconsistência residual.

### C. bcd_encode(major) << 8 | bcd_encode(minor) — build.rs

**Status:** Já protegido (sem alteração necessária).

O `assert!(minor < 10, ...)` em `build.rs:52` já impede minor ≥ 10.
O comentário em build.rs:36-37 documenta a regra:
> minor deve ser 0–9 (cada incremento = 0.10 no BCD). Ao atingir minor 9,
> bumpar major e resetar minor para 0.

Limite implícito: major ≤ 99 (BCD de 2 nibbles). Para SemVer, é um não-problema.

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
