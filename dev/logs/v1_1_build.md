# OpenHOTAS — Build Log V1.1

> **Status final:** Compilou sem erros, sem warnings, Clippy limpo (zero warnings).
> **Data:** Jun/2026
> **Toolchain:** Rust 1.96.0 stable, `thumbv8m.main-none-eabihf`
> **Resultado:** `cargo build --release` + `cargo clippy` — ambos limpos.

---

## Contexto da Versão

A V1.0 compilou com sucesso mas estava baseada em premissas incorretas sobre
o datasheet do MT6826S. O datasheet Rev.1.1 (2024.2, MagnTek) revelou:

- Resolução do encoder é **15-bit**, não 14-bit
- O modo correto de leitura é **Burst Read** (não Single Byte Read)
- A condição de magneto OK é `== 0x00`, não `== 0x02`

Aproveitou-se também para refatorar a estrutura de tasks e otimizar o MCP23S17.

---

## Arquivos Alterados

| # | Arquivo | Ação | Mudança principal |
|---|---|---|---|
| 1 | `src/constants.rs` | Editado | 3 constantes corrigidas: `ANGLE_MAX`, `ANGLE_CENTER`, `CMD_READ_ANGLE` |
| 2 | `src/sensors/mt6826.rs` | Reescrito | Burst Read 6-byte, CRC 3-byte, condição magneto corrigida |
| 3 | `src/sensors/mcp23s.rs` | Editado | `read_reg()` com `blocking_transfer_in_place` (3 bytes contínuos) |
| 4 | `src/tasks/mod.rs` | **Criado** | `pub mod input; pub mod hid; pub mod diagnostic;` |
| 5 | `src/tasks/input.rs` | **Criado** | `input_task` extraída de `main.rs` |
| 6 | `src/tasks/hid.rs` | **Criado** | `usb_task` + `hid_task` extraídas de `main.rs` |
| 7 | `src/tasks/diagnostic.rs` | **Criado** | `diagnostic_task` extraída de `main.rs` |
| 8 | `src/main.rs` | Refatorado | Reduzido a ~120 linhas (init + spawn), tasks removidas |
| 9 | `src/usb/hid_gamepad.rs` | Editado | `usb_task` e `hid_task` removidas (movidas para `tasks/hid.rs`) |
| 10 | `Cargo.toml` | Editado | Adicionado `imagedef-secure-exe` às features do `embassy-rp` |
| 11 | `context/_context.rs` | Editado | Valores 15-bit, protocolo Burst Read |
| 12 | `context/_pinout.rs` | Editado | Protocolo e constantes atualizados |

**Total:** 12 arquivos (4 criados, 8 editados)

---

## Correções Críticas — MT6826S

### Constantes

| Constante | V1.0 | V1.1 | Fonte |
|---|---|---|---|
| `MT6826_CMD_READ_ANGLE` | `0x03` | **`0x0A`** | §8.6.3 — C3-C0 = 1010 (Burst) |
| `MT6826_ANGLE_MAX` | `16383` | **`32767`** | §1, §11 — 15-bit |
| `MT6826_ANGLE_CENTER` | `8192` | **`16384`** | derivado do ANGLE_MAX correto |

### Protocolo SPI

| Aspecto | V1.0 | V1.1 |
|---|---|---|
| Transação | 2 ops: `write(1 byte)` + `read(3 bytes)` | 1 op: `transfer_in_place(6 bytes)` |
| Frame | 3 bytes totais | 6 bytes full-duplex: `[0xA0, 0x03, 0x00, 0x00, 0x00, 0x00]` |
| Dados | `buf[0..2]` | `buf[2..6]` (ANGLE_HI, ANGLE_LO, STATUS, CRC) |
| Cobertura CRC | 2 bytes | 3 bytes (`buf[2..5]`) |
| Máscara CRC | `buf[2] & 0xFE` | `buf[5]` (byte completo, sem máscara) |
| Condição magneto | `(status & 0x06) == 0x02` | `(status & 0x06) == 0x00` |

**Decisão:** Burst Read adotado (não Single Byte Read) porque:
- 1 transação vs 4 no Single Read → menos overhead
- Garante consistência atômica: latch interno no falling edge de CS (§8.6.8)

---

## Otimização MCP23S17

**Problema V1.0:** `read_reg()` fazia `blocking_write([opcode, reg])` + `blocking_read([0x00; 1])`.
Isso inseria gap no clock SCK entre as duas operações.

**Solução V1.1:** Uma única `blocking_transfer_in_place(&mut [opcode, reg, 0x00])` de 3 bytes.
Resultado em `buf[2]`. SCK contínuo, sem overhead de chaveamento.

**Impacto:** `read_chip_raw()` chama `read_reg()` 2× (GPIOA + GPIOB).
Economia de 4 transações → 2 transações por ciclo de leitura de botões.

---

## Refatoração de Tasks

**Problema V1.0:** `main.rs` continha ~175 linhas com as 4 tasks definidas inline.

**Solução V1.1:** Criado `src/tasks/` com 3 módulos.

| Módulo | Conteúdo |
|---|---|
| `tasks/input.rs` | `input_task` — sensores + pipeline + signal |
| `tasks/hid.rs` | `usb_task` + `hid_task` |
| `tasks/diagnostic.rs` | `diagnostic_task` — log a cada 5s |

**Impacto em `main.rs`:**
- Removidos ~55 linhas de corpos de tasks
- `#![allow(static_mut_refs)]` adicionado ao crate root
- `mod tasks;` adicionado
- Spawn atualizado: `tasks::hid::usb_task(...)`, etc.

**Impacto em `usb/hid_gamepad.rs`:**
- `usb_task` e `hid_task` removidas
- `GamepadReport`, `REPORT_SIGNAL`, `axis_to_i16`, `UsbDriver` mantidos

---

## Feature Flag Adicionada

`imagedef-secure-exe` — obrigatória para que o bootloader do RP2350 reconheça
o firmware como executável válido. Ausente na V1.0, adicionada na V1.1.

---

## Decisões Arquiteturais Mantidas da V1.0

Todas as decisões abaixo continuam válidas e **não foram alteradas**:

1. SPI buses via `static mut` + `critical_section` — single-core, seguro
2. MCP23S17 com 2 chips, CS compartilhado — diferenciação via opcode
3. USB buffers como `static mut` — exigência do embassy-usb
4. Deadzone → EMA reset via raw pointer — sound em single-core, sem heap
5. RuntimeStats com `AtomicU32` — suficiente para single-core
6. Flash via `static mut` + `critical_section` — erase antes de write, nunca em ISR

---

## Riscos Técnicos Conhecidos

| Risco | Prob. | Status |
|---|---|---|
| SPI e Flash via `static mut` | — | Aceito — revisar se SMP for adotado |
| Deadzone usando raw pointer para EMA | — | Aceito — sound no contexto atual |
| Burst Read não testado em hardware físico | Alta | **Pendente** — checklist em `02_hardware_specs.md §5` |
| `install_core0_stack_guard()` não implementada | Baixa | API não localizada no embassy-rp 0.10 |

---

## Comandos de Build

```powershell
# Build release
cargo build --release --target thumbv8m.main-none-eabihf

# Clippy (sem --all-targets — não compila para host)
cargo clippy --target thumbv8m.main-none-eabihf

# Flash via probe-rs
probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/openhotas

# Gerar UF2
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/openhotas openhotas.uf2

# Tamanho do binário
cargo size --release --target thumbv8m.main-none-eabihf -- -A
```

---

## Stubs para V2 (dead_code intencional)

| Módulo | Item | Status |
|---|---|---|
| `calibration/data.rs` | `Calibration::start/finish/feed` | `#[allow(dead_code)]` |
| `calibration/cal_store.rs` | `save()` | `#[allow(dead_code)]` |
| `config/settings.rs` | `DeviceConfig::save()`, `active_profile` | `#[allow(dead_code)]` |
| `filters/ema.rs` | `Ema::set_alpha()` | `#[allow(dead_code)]` |
| `filters/response_curve.rs` | interface definida | pass-through |
| `axis/pipeline.rs` | `update_config()` | `#[allow(dead_code)]` |
| `usb/descriptor.rs` | `REPORT_ID_CONFIG = 0x02` | reservado |

---

*OpenHOTAS · Build Log V1.1 · Jun/2026*
