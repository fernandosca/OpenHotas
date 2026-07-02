# OpenHOTAS — Build Log V1.0

> **Status final:** Compilou sem erros. Base funcional estabelecida.
> **Data:** 2026
> **Toolchain:** Rust stable, `thumbv8m.main-none-eabihf`

---

## O que foi feito

Build inicial do firmware. Estrutura de módulos criada, pipeline de sinal
implementado, stack USB HID configurada, sensores com drivers preliminares.

| # | Arquivo | Ação |
|---|---|---|
| 1 | `src/main.rs` | Setup inicial com tasks inline (~175 linhas) |
| 2 | `src/constants.rs` | Fonte única de constantes (valores V1.0 — alguns incorretos) |
| 3 | `src/spi_bus.rs` | Compartilhamento SPI via `static mut` + `critical_section` |
| 4 | `src/sensors/mt6826.rs` | Driver MT6826S — Single Byte Read (protocolo incorreto) |
| 5 | `src/sensors/mcp23s.rs` | Driver MCP23S17 — `blocking_write` + `blocking_read` separados |
| 6 | `src/calibration/data.rs` | `CalibrationData` com constantes 14-bit (incorretas) |
| 7 | `src/filters/` | MaxJump, EMA, Deadzone, Expo, ResponseCurve |
| 8 | `src/axis/pipeline.rs` | `AxisPipeline` — ordem do pipeline definida |
| 9 | `src/config/settings.rs` | `DeviceConfig` — load/save com CRC32 |
| 10 | `src/storage/flash.rs` | Primitivas de flash — erase, write, read, crc32 |
| 11 | `src/usb/` | HID descriptor + `GamepadReport` + `REPORT_SIGNAL` |

---

## Decisões Arquiteturais Tomadas

Todas continuam válidas em V1.1+:

1. **SPI compartilhado via `static mut` + `critical_section`** — sound em single-core.
2. **MCP23S17 com 2 chips, CS compartilhado** — diferenciação via opcode.
3. **USB buffers como `static mut`** — exigência de lifetime `'static` do embassy-usb.
4. **Deadzone → EMA reset via raw pointer** — sound em single-core, sem heap.
5. **RuntimeStats com `AtomicU32`** — suficiente para single-core.
6. **Flash via `static mut` + `critical_section`** — erase antes de write, nunca em ISR.
7. **Tasks como closures inline em `main.rs`** — refatorado na V1.1.

---

## Erros Identificados (corrigidos na V1.1)

| # | Erro | Causa | Impacto |
|---|---|---|---|
| 1 | `ANGLE_MAX = 16383` | Datasheet interpretado como 14-bit | Range de ângulo errado |
| 2 | `ANGLE_CENTER = 8192` | Derivado do ANGLE_MAX errado | Centro de calibração errado |
| 3 | `MT6826_CMD = 0x03` | Comando Read Register genérico | Protocolo errado — não usa Burst |
| 4 | Frame SPI de 3 bytes | Single Byte Read em vez de Burst | Dados incompletos, sem CRC |
| 5 | CRC sobre 2 bytes | Cobertura incorreta | Validação falha |
| 6 | `check_magnet == 0x02` | Interpretação invertida do STATUS | Rejeita sensor saudável |
| 7 | `blocking_write + blocking_read` no MCP | 2 transações com gap no SCK | Overhead desnecessário |
| 8 | Tasks em `main.rs` | Sem diretório `src/tasks/` | `main.rs` com ~175 linhas |

---

## Riscos Técnicos Aceitos

| Risco | Condição de segurança |
|---|---|
| SPI via `static mut` | Single-core |
| Flash via `static mut` | Single-core, nunca em ISR |
| Deadzone raw pointer para EMA | Single-core, sem concorrência no pipeline |
| `transmute` de lifetimes | Inicialização única em `main.rs` |

---

*OpenHOTAS · Build Log V1.0*
