# OpenHOTAS — Pinout & Configuração de Hardware

> **LEIA ANTES DE IMPLEMENTAR QUALQUER DRIVER.**
> Mapeamento físico definitivo. Não alterar sem atualizar o hardware e o dev/logs/.

---

## 1. Visão Geral

| Componente | Especificação |
|---|---|
| Chip | RP2350 |
| Placa | Raspberry Pi Pico 2 (header 2×20) |
| Flash interna | 2MB via barramento QSPI |
| USB | Device Full-Speed nativo — classe HID Gamepad |

### Barramentos SPI

| Barramento | Periférico(s) | Função |
|---|---|---|
| **SPI0** | 2× MCP23S17 | Leitura de 32 botões |
| **SPI1** | 3× MT6826S | Leitura de 3 eixos absolutos (X, Y, Twist) |

> ⚠️ **REGRA DE ISOLAMENTO INVIOLÁVEL:**
> SPI0 e SPI1 são completamente independentes e exclusivos.
> Nunca misture periféricos entre eles.

---

## 2. SPI0 — Expansão de Botões (2× MCP23S17)

| Função | GPIO | Pino Físico | Descrição |
|---|---|---|---|
| SPI0_RX | GP4 | Pino 6 | MISO — dados dos MCP23S17 |
| SPI0_CSn | GP5 | Pino 7 | CS compartilhado (diferenciação por opcode) |
| SPI0_SCK | GP6 | Pino 9 | Clock SPI0 |
| SPI0_TX | GP7 | Pino 10 | MOSI — comandos para MCP23S17 |
| INT | GP8 | Pino 11 | Interrupção Wired-OR (ambos os chips) |

### Configuração Embassy (`main.rs`)

```rust
let mut spi0_cfg = SpiConfig::default();
spi0_cfg.frequency = 1_000_000;
let spi0 = Spi::new_blocking(p.SPI0, p.PIN_6, p.PIN_7, p.PIN_4, spi0_cfg);
//                                      SCK      MOSI     MISO
spi_bus::init_spi0(unsafe { core::mem::transmute(spi0) });

let cs_mcp = Output::new(p.PIN_5, Level::High); // CS ativo em LOW
```

### CS Compartilhado — Endereçamento por Opcode

Os dois MCP23S17 compartilham GP5. A distinção é feita via bits de endereço
no opcode SPI: `opcode = 0b0100_[A2][A1][A0][R/W̄]`

| Chip | A2:A1:A0 | Write | Read | Botões |
|---|---|---|---|---|
| U1 | 000 | `0x40` | `0x41` | B0..B15 |
| U2 | 001 | `0x42` | `0x43` | B16..B31 |

> Os pinos A0, A1, A2 de cada chip devem estar soldados ao GND ou VCC
> conforme a tabela acima. O firmware não pode alterá-los.

### Linha de Interrupção (GP8)

- Saídas INT de U1 e U2 interligadas em **Wired-OR** → GP8
- GP8 configurado como entrada com **pull-up interno ativo**
- Borda de descida (LOW) = algum botão mudou de estado
- A linha é limpa automaticamente ao ler GPIOA/GPIOB via SPI
- ⚠️ Requer que as saídas INT dos chips operem em **open-drain**

---

## 3. SPI1 — Encoders de Eixo (3× MT6826S)

| Função | GPIO | Pino Físico | Descrição |
|---|---|---|---|
| SPI1_RX | GP12 | Pino 16 | MISO — dados dos MT6826S |
| SPI1_SCK | GP14 | Pino 19 | Clock SPI1 |
| SPI1_TX | GP15 | Pino 20 | MOSI — comandos para MT6826S |
| CS — Eixo X | GP10 | Pino 14 | Chip Select encoder X |
| CS — Eixo Y | GP13 | Pino 17 | Chip Select encoder Y |
| CS — Eixo Twist | GP16 | Pino 21 | Chip Select encoder Twist |

### Configuração Embassy (`main.rs`)

```rust
let mut spi1_cfg = SpiConfig::default();
spi1_cfg.frequency = MT6826_SPI_FREQ_HZ; // 1_000_000
spi1_cfg.polarity = Polarity::IdleHigh;           // CPOL=1
spi1_cfg.phase = Phase::CaptureOnSecondTransition; // CPHA=1 → Mode 3
let spi1 = Spi::new_blocking(p.SPI1, p.PIN_14, p.PIN_15, p.PIN_12, spi1_cfg);
//                                      SCK       MOSI      MISO
spi_bus::init_spi1(unsafe { core::mem::transmute(spi1) });

let sens_x = Mt6826::new(Output::new(p.PIN_10, Level::High)); // CS X
let sens_y = Mt6826::new(Output::new(p.PIN_13, Level::High)); // CS Y
let sens_t = Mt6826::new(Output::new(p.PIN_16, Level::High)); // CS Twist
```

> ⚠️ `embassy_rp` não exporta `MODE_3`. Configurar manualmente via
> `Polarity::IdleHigh` + `Phase::CaptureOnSecondTransition`.

### CS Dedicado por Encoder

Cada MT6826S tem seu próprio pino CS. A sequência de transação é:
1. `cs.set_low()` — ativa o chip (latch interno do ângulo)
2. `spi.blocking_transfer_in_place(&mut buf)` — 6 bytes full-duplex
3. `cs.set_high()` — desativa o chip

---

## 4. USB

| Função | Observação |
|---|---|
| USB D+ / D- | Pinos físicos nativos do RP2350 |
| Classe | HID Gamepad (Report ID 0x01) |
| Polling rate | 1ms |
| VID/PID | `0x16C0` / `0x27DB` |
| Manufacturer | `"OpenHOTAS"` |
| Product | `"OpenHOTAS Gamepad"` |

---

## 5. Flash Interna

| Parâmetro | Valor |
|---|---|
| Tamanho | 2MB |
| Interface | QSPI (gerenciada pelo RP2350) |
| Setor | 4096 bytes |
| Slot A (config) | `STORED_V2_SLOT_A` = `FLASH_SIZE - SECTOR_SIZE` (0x1FF000) |
| Slot B (backup) | `STORED_V2_SLOT_B` = `FLASH_SIZE - 2 * SECTOR_SIZE` (0x1FE000) |

> A persistência usa double-buffer com geração (power-fail safety). Boot lê
> ambos os slots, usa o de maior geração. Save escreve no slot inativo.
> Ver layout detalhado em `dev/context/04_software_contracts.md §9`.

> Offsets são **relativos ao início da flash física (0x00)**, não ao XIP base (0x10000000).

---

## 6. Pinos Não Utilizados / Reservados

Todos os GPIOs não listados acima estão livres para expansão futura.
Não há conflito de função alternativa nos pinos usados.

---

*OpenHOTAS · Pinout V1.4 · Jul/2026*
