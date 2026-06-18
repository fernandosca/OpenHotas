# OpenHOTAS — Especificações de Hardware & Protocolos SPI

> **Fonte:** Datasheet MT6826S Rev.1.1 (2024.2, MagnTek) + validação V1.21
> Contrato fixo — não alterar sem re-testar em hardware e atualizar o log.

---

## 1. MT6826S — Encoder Magnético Absoluto 15-bit

<!--
PREMISSAS MECÂNICAS
- Magneto diametralmente magnetizado
- Alimentação dos encoders em 3.3V
- Um CS dedicado por encoder
- Zero Position EEPROM não utilizado em operação normal
-->

### Identidade

| Parâmetro | Valor |
|---|---|
| Fabricante | MagnTek |
| Resolução | **15-bit** — ANGLE[14:0] |
| Range do ângulo | `0 .. 32767` (2¹⁵ − 1) |
| Centro nominal | `16384` (2¹⁴ — midpoint de [0..32767]) |
| Interface | SPI 4-Wire, **Mode 3** (CPOL=1, CPHA=1) |
| Frequência SPI validada | **1 MHz** (conservadora; máximo do chip: 15.6 MHz) |

> ⚠️ O datasheet §1 confirma: **"15-Bit Core Resolution"**.

### Premissas de Integração

- Barramento SPI compartilhado entre encoders
- Frequência SPI validada em 1 MHz
- Leitura exclusivamente via Burst Read
- Calibração de centro realizada por software

### Constantes (fonte única: `constants.rs`)

```rust
pub const MT6826_SPI_FREQ_HZ:       u32 = 1_000_000;
pub const MT6826_SPI_MODE:          u8  = 3;        // CPOL=1, CPHA=1
pub const MT6826_CMD_READ_ANGLE:    u8  = 0x0A;     // Burst Read — C3-C0 = 1010
pub const MT6826_CRC8_POLY:         u8  = 0x07;     // X⁸ + X² + X + 1
pub const MT6826_STATUS_ERROR_MASK: u8  = 0x06;     // bits[1] e [2] do STATUS
pub const MT6826_ANGLE_SHIFT:       u8  = 1;        // remove LSB fixo zero
pub const MT6826_ANGLE_MAX:         u16 = 32767;    // 15-bit max
pub const MT6826_ANGLE_CENTER:      u16 = 16384;    // centro nominal pré-calibração
```

---

## 2. MT6826S — Mapa de Registradores Relevantes

Fonte: datasheet §11.

| Endereço | Conteúdo | Tipo |
|---|---|---|
| `0x003` | ANGLE[14:7] — 8 bits MSB do ângulo | Read Only |
| `0x004` | ANGLE[6:0] + **LSB fixo = 0** | Read Only |
| `0x005` | STATUS[2:0] — warnings críticos | Read Only |
| `0x006` | CRC[7:0] | Read Only |

### Byte STATUS (0x005)

| Bit | Nome | Warning ativo quando |
|---|---|---|
| [0] | Rotation Over Speed | = 1 |
| [1] | Weak Magnetic Field | = 1 |
| [2] | Under Voltage | = 1 |
| [7:3] | Fixed 00000 | — |

**Condição magneto OK:** `(status & 0x06) == 0x00`

### Extração do Ângulo 15-bit

```
raw_u16 = (buf[2] as u16) << 8 | (buf[3] as u16)
angle   = raw_u16 >> 1     // remove LSB fixo zero
```

---

## 3. MT6826S — Protocolo Burst Read

Fonte: datasheet §8.6.8.

O Burst Read é o **único modo de leitura usado no OpenHOTAS**. Uma única transação SPI lê os 4 registradores (0x003..0x006) em snapshot atômico — o chip faz latch interno no falling edge do CS e não atualiza até todos os registradores serem lidos.

### Frame Completo — 6 bytes full-duplex

```
Byte #   MOSI        MISO
──────────────────────────────────────────────
  0      0xA0        garbage
  1      0x03        garbage
  2      0x00        Reg 0x003 = ANGLE[14:7]
  3      0x00        Reg 0x004 = ANGLE[6:0] + LSB=0
  4      0x00        Reg 0x005 = STATUS[2:0]
  5      0x00        Reg 0x006 = CRC[7:0]
```

Bytes MOSI após o comando são dummy — o sensor os ignora.

### Construção do Comando

```
C3-C0 = 1010 (0x0A = Burst Angle Read)
Endereço = 0x003
→ Byte 0 MOSI = 0xA0
→ Byte 1 MOSI = 0x03
```

### Implementação Rust

```rust
fn read(&mut self) -> Result<u16, SensorError> {
    spi_bus::with_spi1(|spi| {
        self.cs.set_low();

        let mut buf = [0xA0u8, 0x03, 0x00, 0x00, 0x00, 0x00];
        spi.blocking_transfer_in_place(&mut buf)
           .map_err(|_| SensorError::SpiError)?;

        self.cs.set_high();

        // CRC cobre exatamente 3 bytes: buf[2], buf[3], buf[4]
        let crc_expected = Self::compute_crc8(&buf[2..5]);
        if crc_expected != buf[5] {
            self.error_count = self.error_count.saturating_add(1);
            self.last_healthy = false;
            return Err(SensorError::CrcError);
        }

        // Magneto OK = (status & 0x06) == 0x00
        if !Self::check_magnet(buf[4]) {
            self.last_healthy = false;
            return Err(SensorError::MagnetError);
        }

        let raw: u16 = (buf[2] as u16) << 8 | buf[3] as u16;
        let angle = raw >> MT6826_ANGLE_SHIFT; // >> 1

        self.last_healthy = true;
        Ok(angle.min(MT6826_ANGLE_MAX))
    })
}

fn compute_crc8(data: &[u8]) -> u8 {
    let mut crc: u8 = 0;
    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if crc & 0x80 != 0 {
                crc = (crc << 1) ^ MT6826_CRC8_POLY; // 0x07
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

fn check_magnet(status: u8) -> bool {
    (status & MT6826_STATUS_ERROR_MASK) == 0x00
}
```

---

## 3.1 MT6826S — Engineering Notes & Design Rules

### Atomic Snapshot Guarantee

Quando `CSN` é colocado em LOW, o MT6826S faz latch interno dos registradores `0x003..0x006` e congela os dados até que toda a leitura seja concluída:

- ANGLE, STATUS e CRC pertencem exatamente à mesma amostra física
- Não existem condições de corrida entre ângulo e CRC
- A leitura deve ocorrer em uma única transação SPI
- **Nunca dividir Burst Read em múltiplas transações**

### SPI Timing Requirements

| Parâmetro | Valor |
|---|---|
| TL | 100 ns |
| TSCK | 64 ns |
| TSCKH | 30 ns |
| TSCKL | 30 ns |
| TH | 0.5 × TSCK |

Frequência máxima teórica: `1 / 64ns ≈ 15.6 MHz`

O OpenHOTAS valida `MT6826_SPI_FREQ_HZ = 1_000_000` — margem superior a 15× em relação ao limite do componente.

### CRC Characteristics

CRC aplicado sobre os registradores `0x003`, `0x004` e `0x005` (24 bits total).

| Parâmetro | Valor |
|---|---|
| Polynomial | x⁸ + x² + x + 1 |
| Poly | 0x07 |
| Init | 0x00 |
| Bit Order | MSB First |
| First Bit | ANGLE[14] |

Qualquer divergência entre CRC calculado e CRC recebido invalida toda a amostra.

### STATUS Register Interpretation

| Bit | Nome | Significado |
|---|---|---|
| 0 | Overspeed | Velocidade acima da capacidade interna |
| 1 | Weak Magnet | Campo magnético insuficiente |
| 2 | Under Voltage | Alimentação abaixo do limite |

Condição saudável: `(status & 0x06) == 0x00`

O firmware valida Weak Magnet e Under Voltage. O bit Overspeed é reservado para diagnóstico futuro.

### Runtime Diagnostics

Todo firmware deve manter contadores independentes para:

- CRC Errors
- Weak Magnet Warnings
- Under Voltage Warnings
- Overspeed Warnings

### Interpretação dos Erros

**CRC Error crescente** — causas de software/elétrica: EMI, aterramento inadequado, alimentação instável, cabo longo, solda defeituosa, conectores intermitentes.

**Under Voltage** — causas: queda de alimentação, cabo USB inadequado, regulador instável, consumo excessivo do sistema.

**Overspeed** — causas: velocidade de rotação acima da capacidade interna do encoder.

<!--
INTERPRETAÇÃO MECÂNICA DOS ERROS
- Weak Magnet: magneto distante, magneto inadequado, desalinhamento mecânico, air gap excessivo
- CRC crescente pode também indicar vibração mecânica causando intermitência no conector
-->

### Power-Up Behavior

Após energização, `TPwrUp típico ≈ 3 ms`. O encoder bloqueia saídas internas durante este período.

- Aguardar boot completo antes de usar leituras críticas
- Descartar primeiras amostras após power-on

### Zero Position Strategy

O MT6826S suporta redefinição permanente do ângulo zero via registradores `0x009` e `0x00A` (`ZERO_POS[11:0]`). O OpenHOTAS **não utiliza** esta funcionalidade — a centralização ocorre por software durante a inicialização.

<!--
ZERO POSITION — NOTA MECÂNICA
- EEPROM interna: endurance típica de 1000 ciclos
- Não usar gravação frequente para calibração dinâmica
- Evitar desgaste: centralização por software é o padrão adotado
- Magneto diametral Ø10 mm × 2.5 mm recomendado pelo fabricante
- Campo válido: 30 mT .. 1000 mT
- Air Gap recomendado: 1.0 mm .. 3.0 mm
- Desalinhamento máximo: 0.3 mm
- Evitar estruturas ferromagnéticas próximas ao sensor
-->

---

## 4. MT6826S — Checklist de Verificação em Hardware

Executar na primeira vez que o firmware rodar no hardware físico:

- [ ] Ângulo varia entre 0 e ~32767 ao girar 360°
- [ ] Nenhum `CrcError` em operação normal
- [ ] `check_magnet()` retorna `true` com magneto posicionado
- [ ] `raw >> 1` produz [0, 32767] — não [0, 65535]
- [ ] 4 bytes de uma transação pertencem ao mesmo snapshot (latch no falling edge de CS)

<!--
CHECKLIST MECÂNICA
- Valor próximo de 16384 na posição central mecânica (depende de montagem física)
- Air gap uniforme durante toda a rotação
- Magneto centrado sobre o sensor sem inclinação angular
-->

---

## 5. MCP23S17 — Expansor de I/O

### Identidade

- 2 chips compartilham o mesmo CS físico (GP5)
- Diferenciação via campo de endereço no opcode SPI
- 16 pinos de I/O por chip → 32 botões totais (2 × 16)
- Debounce por software: 3 amostras estáveis consecutivas (`MCP23S17_DEBOUNCE_COUNT`)

### Endereçamento por Opcode

| Chip | A2:A1:A0 | Opcode Write | Opcode Read | Botões |
|---|---|---|---|---|
| U1 | 000 | `0x40` | `0x41` | B0..B15 |
| U2 | 001 | `0x42` | `0x43` | B16..B31 |

Fórmula: `opcode = 0b0100_[A2][A1][A0][R/W̄]`

### Registradores Usados

```rust
pub const MCP23S17_IODIRA: u8 = 0x00; // direção port A (0xFF = tudo input)
pub const MCP23S17_IODIRB: u8 = 0x01;
pub const MCP23S17_IOCON:  u8 = 0x0A; // config: HAEN=1, ODR=1
pub const MCP23S17_GPPUA:  u8 = 0x0C; // pull-up port A (0xFF = todos ativos)
pub const MCP23S17_GPPUB:  u8 = 0x0D;
pub const MCP23S17_GPIOA:  u8 = 0x12; // leitura port A
pub const MCP23S17_GPIOB:  u8 = 0x13; // leitura port B
```

### Leitura Otimizada — `blocking_transfer_in_place`

```rust
// ✅ 1 transação, SCK contínuo
fn read_reg(&mut self, addr: u8, reg: u8) -> Result<u8, SensorError> {
    let opcode = read_opcode(addr);
    spi_bus::with_spi0(|spi| {
        self.cs.set_low();
        let mut buf = [opcode, reg, 0x00];
        spi.blocking_transfer_in_place(&mut buf)
           .map_err(|_| SensorError::SpiError)?;
        self.cs.set_high();
        Ok(buf[2]) // dado válido em buf[2]
    })
}

// ❌ 2 transações separadas com gap no SCK — não usar
// spi.blocking_write(&[opcode, reg])
// spi.blocking_read(&mut [0x00; 1])
```

### Sequência de Inicialização (cold boot)

Para cada chip (U1 e U2):

1. `IOCON = 0x0C` — habilita HAEN (decodificação de endereço) + ODR (open-drain INT)
2. `IODIRA = 0xFF`, `IODIRB = 0xFF` — todos os pinos como entrada
3. `GPPUA = 0xFF`, `GPPUB = 0xFF` — pull-ups internos ativados

### Output de Leitura

```
merged_u32 = (chip1.state as u32) << 16 | (chip0.state as u32)
```

Botões ativos em nível baixo (pull-up + botão a GND).

---

## 6. Compartilhamento de Barramento SPI (`spi_bus.rs`)

Embassy exige ownership exclusivo dos periféricos SPI. A solução é um wrapper global acessado via closure de escopo seguro.

```rust
// Padrão obrigatório para qualquer acesso ao SPI
spi_bus::with_spi1(|spi| {
    // uso do spi dentro desta closure
});
```

**Regra:** Sound **apenas** em single-core — revisar se DMA ou SMP forem adotados.

---

## Observação Final

Este documento é a fonte de verdade para protocolo SPI, requisitos elétricos, critérios de validação, diagnósticos de campo e integração com o firmware OpenHOTAS.

Qualquer alteração deve ser acompanhada de validação em hardware e atualização da documentação.

---

*OpenHOTAS · Hardware Specs V1.21 · Jun/2026*
*Fonte: MagnTek MT6826S Datasheet Rev.1.1 (2024.2)*
