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

## 8. Crate `openhotas-filters` — Extração de Lógica Pura (Jun/2026)

### Motivo

O firmware é `[[bin]]` com `test = false`. Extrair lógica pura (filtros, calibração,
crc32) para um crate library separado permite testes unitários no host sem
dependências de HAL/embassy.

### Novo Crate

```
crates/openhotas-filters/
├── Cargo.toml          # deps: libm
└── src/
    ├── lib.rs          # #![no_std] + re-exports
    ├── tuning.rs       # 3 constantes de default
    ├── ema.rs          # Filtro EMA (6 testes)
    ├── deadzone.rs     # Zona morta (5 testes)
    ├── max_jump.rs     # Rejeição de spikes (4 testes)
    ├── response_curve.rs # Curva piecewise (4 testes)
    ├── calibration.rs  # CalibrationData + Calibration (6 testes)
    └── crc32.rs        # CRC-32 ISO-HDLC (4 testes)
```

### Dependências

| Dep | Tipo | Necessidade |
|-----|------|-------------|
| `libm` | Runtime | `fabsf()` em deadzone e max_jump |
| `embassy-*` | — | **Não usado** |
| `cortex-m` | — | **Não usado** |
| `openhotas-protocol` | — | **Não usado** |

### O que migrou

| De | Para | Observação |
|----|------|------------|
| `firmware/src/filters/ema.rs` | `crates/.../src/ema.rs` | Import: `crate::tuning::*` |
| `firmware/src/filters/deadzone.rs` | `crates/.../src/deadzone.rs` | Import: `crate::tuning::*` |
| `firmware/src/filters/max_jump.rs` | `crates/.../src/max_jump.rs` | Import: `crate::tuning::*` |
| `firmware/src/filters/response_curve.rs` | `crates/.../src/response_curve.rs` | Sem imports |
| `firmware/src/calibration/data.rs` | `crates/.../src/calibration.rs` | Struct + `Default` + `apply()` |
| `firmware/src/storage/flash.rs` (crc32) | `crates/.../src/crc32.rs` | Função standalone |

### Re-exports no Firmware

- `firmware/src/filters/mod.rs` → re-exporta tipos do novo crate
- `firmware/src/calibration/mod.rs` → re-exporta `CalibrationData`, `Calibration`
- `firmware/src/storage/flash.rs` → re-exporta `crc32`

Nenhuma mudança de import necessária nos consumidores (`pipeline.rs`, `config/runtime.rs`,
`main.rs`).

### Testes — 29 total

| Módulo | Testes | Destaques |
|--------|--------|-----------|
| EMA | 6 | convergência, alpha zero/um, reset |
| Deadzone | 5 | remap, flag just_entered, zero threshold |
| MaxJump | 4 | spike positivo e negativo |
| ResponseCurve | 4 | linear, clamp, interpolação, set_points |
| Calibration | 6 | center, min, max, degenerate, assimétrico |
| CRC32 | 4 | vetor ISO-HDLC, byte único, consistência |

### Gate de Qualidade

```
openhotas-filters tests  : PASS (29/29)
openhotas-filters clippy : PASS (zero warnings)
Firmware build           : PASS
Firmware clippy (cross)  : PASS (zero warnings)
Fmt                      : PASS
cargo tree (host)        : libm only — zero embedded deps
```

---

## 9. Validação em Hardware — Raspberry Pi Pico 2 / RP2350A (26/Jun/2026)

### Sintoma inicial

O UF2 era aceito pelo modo `BOOTSEL`, mas, após reiniciar, o dispositivo não
era enumerado pelo host como HID.

### Causa raiz

O firmware selecionava corretamente o alvo `thumbv8m.main-none-eabihf` e o
chip `rp235xa`, porém `firmware/memory.x` ainda mantinha o layout de RP2040:

- reserva `BOOT2` de 256 bytes em `0x10000000`;
- início da aplicação em `0x10000100`;
- `.start_block` do RP2350 posicionado fora da janela inicial examinada pela
  Boot ROM.

O RP2350 não usa a reserva `BOOT2` do RP2040. A imagem precisa iniciar em
`0x10000000`, com a definição `.start_block` dentro dos primeiros 4 KiB da
flash.

### Correção

- removida a região `BOOT2` de `firmware/memory.x`;
- `FLASH` alterada para `ORIGIN = 0x10000000`;
- `.start_block` inserido imediatamente após `.vector_table`;
- `_stext` movido para depois de `.start_block`;
- UF2 identificado como família `RP2350_ARM_S` (`0xe48bff59`).

Layout confirmado no ELF:

```text
.vector_table  0x10000000
.start_block   0x10000114
.text          0x10000128
```

### Resultado em hardware

**PASS:** após gravar o UF2 via USB/`BOOTSEL`, o Raspberry Pi Pico 2 iniciou o
firmware e foi identificado corretamente pelo host como dispositivo HID.

### Detecção de periféricos desconectados

O primeiro teste sem sensores revelou dois falsos positivos:

- o MISO flutuante do MT6826S produzia frames zerados cujo CRC também era zero;
- escritas SPI no MCP23S17 retornavam sucesso mesmo sem nenhum chip conectado.

Correções aplicadas:

- pull-up interno habilitado em `SPI0_MISO` e `SPI1_MISO` depois da
  inicialização do periférico SPI (o Embassy limpa os pulls ao configurar o
  pad);
- inicialmente, frames MT6826S totalmente zerados foram rejeitados como
  `NotPresent`; essa regra foi corrigida posteriormente na Seção 12 porque
  ângulo zero, status zero e CRC zero formam uma resposta válida;
- inicialização do MCP23S17 lê de volta `IOCON`, `IODIR` e `GPPU` nos dois
  endereços e exige os valores configurados;
- novo erro interno `SensorError::NotPresent` diferencia ausência física de
  falha de transporte.

Validação física sem MT6826S e MCP23S17 conectados:

```text
Sensor X:      UNHEALTHY
Sensor Y:      UNHEALTHY
Sensor Twist:  UNHEALTHY
Raw X/Y/Twist: 16384 (fallback central)
Buttons:       DEGRADED
Protocol CRC:  0
Flash errors:  0
```

**PASS:** HID e CDC permaneceram funcionais durante a degradação, sem expor
leituras falsas como sensores saudáveis.

---

## 10. Calibração Circular dos Eixos (28/Jun/2026)

### Motivação

Durante a montagem do segundo eixo, o curso físico do ímã atravessou o rollover
do MT6826S (`32767 -> 0`). A calibração linear anterior exigia
`min_raw < center_raw < max_raw`, portanto não conseguia representar esse
curso sem reposicionar mecanicamente o ímã.

### Solução

A calibração passou a operar no domínio circular de 15 bits. Cada amostra e
cada extremo são representados pela menor distância assinada em relação ao
centro:

```rust
delta = ((raw - center + 16384) mod 32768) - 16384
```

Com isso:

- `min_raw`, `center_raw` e `max_raw` continuam sendo os três pontos capturados;
- a ordem numérica dos pontos brutos deixa de ser relevante;
- o curso pode atravessar `32767 -> 0` sem salto na saída processada;
- sensores montados nas duas orientações são aceitos;
- cada extremo deve estar em um lado diferente do centro;
- o curso mínimo de 1000 contagens continua obrigatório;
- `max_jump_raw` passa a ser escalado pelo curso circular calibrado.

Não foi adicionada migração de calibração persistida. Esta mudança será testada
partindo de configuração limpa e nova captura dos três pontos.

### Testes automatizados

Foram adicionados casos para:

- passagem pelo zero;
- passagem pelo zero com direção invertida;
- centro e extremos corretos;
- rejeição de extremos capturados no mesmo lado do centro.

Resultado:

```text
openhotas-filters:  32/32 PASS
openhotas-protocol:  8/8 PASS
firmware release:        PASS
```

### Validação física parcial

- Eixo X validado em SPI Mode 3 a 1 MHz, com centro e limites físicos corretos
  no painel de joystick do Windows e zero erros após movimento completo.
- Eixo Y detectado de forma estável e confirmou em dados brutos a passagem
  física pelo rollover (`~30372 -> 32767 -> 0 -> ~2085`).
- A calibração circular do Y foi salva; a validação prolongada da saída
  processada com os três eixos ainda está pendente.
- Erros globais elevados durante estes testes são produzidos principalmente
  pelos canais X/Twist desconectados, consultados continuamente pelo firmware.

### Pendência observada

O eixo Y apresentou raros erros CRC apenas ao mover pelo lado que atravessa o
rollover. Parado, ou movido no lado que não atravessa zero, o contador não
aumentou. A calibração circular não participa da aquisição SPI nem do cálculo
do CRC, portanto a causa ainda precisa ser instrumentada no driver.

Investigação futura:

- registrar `ANGLE_H`, `ANGLE_L`, `STATUS`, CRC recebido e CRC calculado;
- registrar amostra anterior/atual nas falhas próximas ao rollover;
- não adicionar retry antes de capturar a causa, para evitar mascará-la;
- classificar resposta totalmente `0xFF` como `NotPresent`, separando sensor
  ausente de erro CRC real.

### Próxima etapa

Conectar X, Y e Twist simultaneamente, recalibrar os três eixos a partir de
estado limpo e executar testes prolongados de movimento, centro, extremos,
rollover, CRC e enumeração HID.

---

## 11. Proteção de troca de CS no barramento MT6826S (29/Jun/2026)

### Sintoma observado

Após operação prolongada com os três MT6826S no mesmo SPI1, os três eixos
passaram simultaneamente para `UNHEALTHY`. Os contadores por eixo cresceram na
mesma taxa e mantiveram valores idênticos, enquanto os contadores globais de
CRC e magneto permaneceram zerados.

Naquele momento, o comportamento foi interpretado como resposta zerada
persistente no MISO compartilhado. A investigação posterior identificou o
conector frouxo e corrigiu a classificação `NotPresent`, conforme a Seção 12.
Como o pipeline usa o centro nominal quando uma leitura falha, o joystick
deixou de responder aos movimentos enquanto a falha permaneceu ativa.

Um dos módulos foi isolado como suspeito de manter o barramento ocupado. Em
teste individual posterior, ele voltou a responder normalmente. Portanto, não
há confirmação de dano permanente; o reteste com os três módulos reunidos
permanece necessário.

### Margem entre sensores — tentativa posteriormente removida

As leituras de X, Y e Twist eram executadas consecutivamente, sem tempo morto
explícito entre a subida do CSN de um sensor e a descida do CSN seguinte.

Foi testado um intervalo conservador de 2 us imediatamente após cada
`CSN HIGH`. O teste não alterou a falha observada e foi removido na revisão de
1/Jul/2026, pois o datasheet não exige tempo morto depois de `CSN HIGH`.

O datasheet indica que MISO retorna para alta impedância na subida de CSN e não
exige espera em escala de microssegundos. O intervalo adicional serve como
margem para propagação e fiação no barramento físico; ele não recupera um
dispositivo que ignore CSN por falha elétrica.

Impacto temporal por ciclo completo:

```text
3 sensores x 2 us = 6 us
periodo do input task = 500 us
```

### Validação

```text
Firmware release: PASS
Sensor suspeito isolado: voltou a responder
Tres sensores simultaneos: falha reproduzida; causa física confirmada na Seção 12
```

### Diagnóstico pendente

O comando `sensor-status` contabiliza respostas `NotPresent` no total por eixo,
mas o comando `errors` ainda não inclui `NotPresent` no total global. Por isso,
uma falha contínua de aquisição pode coexistir com CRC e magneto zerados. Essa
limitação de telemetria deve ser corrigida separadamente, sem confundir estado
atual de saúde com o histórico de falhas desde o boot.

---

## 12. Revisão final da aquisição MT6826S e toolchain RP2350 (1/Jul/2026)

### Causa raiz da falha simultânea dos sensores

Durante os testes, X, Y e Twist acumulavam erros na mesma taxa e alternavam
para `UNHEALTHY`. Foram testados intervalos entre CS e frequências SPI menores,
sem eliminar a falha. A redução para 250 kHz ultrapassou o orçamento da tarefa
de entrada de 500 us e impediu o agendamento normal do HID.

A inspeção física confirmou a causa raiz: **conector frouxo no barramento dos
sensores**. Após corrigir o conector, a comunicação voltou a operar. Portanto,
o problema principal não era o algoritmo CRC, o comando burst ou o modo SPI.

### Revisão contra o datasheet MT6826S Rev.1.1

Foram confirmados:

- SPI Mode 3 (`CPOL=1`, `CPHA=1`);
- burst read `0xA0 0x03` seguido de quatro bytes dummy;
- CRC-8 com polinômio `0x07`, init zero e ordem MSB first;
- CRC cobrindo `ANGLE_H`, `ANGLE_L` e `STATUS`;
- extração do ângulo de 15 bits com deslocamento de um bit.

Problemas encontrados e soluções permanentes:

1. O driver não garantia explicitamente `TL` entre `CSN LOW` e o primeiro
   clock. Foi adicionado setup de 1 us (`TL` mínimo do datasheet: 100 ns).
2. O driver subia CS imediatamente após a transferência, sem garantir `TH`.
   Foi adicionado hold de 1 us antes de `CSN HIGH` (`TH` mínimo em 1 MHz:
   0,5 us).
3. Não havia espera explícita de power-up. A tarefa agora aguarda 5 ms uma
   única vez antes da primeira leitura (`TPwrUp` típico: 3 ms).
4. A resposta `00 00 00 00` era classificada como sensor ausente, apesar de
   representar legalmente ângulo zero com CRC zero. Com pull-up no MISO, sensor
   ausente passa a ser identificado por `FF FF FF FF`.
5. O atraso experimental de 2 us após `CSN HIGH` foi removido. Permanecem
   somente os tempos exigidos pelo datasheet.

A frequência SPI final voltou para 1 MHz. A instrumentação temporária
`sensor-frames` foi removida após a causa física ser identificada, mantendo o
protocolo em v3.0.

### Correções no fluxo ELF/UF2

O conversor `elf2uf2-rs` gerou UF2 sem a correção do errata RP2350-E10 e foi
removido do ambiente, documentação e CI. O fluxo oficial passou a usar:

```sh
picotool uf2 convert firmware.elf -t elf firmware.uf2 -t uf2 --abs-block
```

O `picotool` local foi validado como build Release oficial, versão 2.1.0, com
suporte a `uf2 convert --abs-block`. O CI usa pacote oficial 2.2.0-a4.

Todos os artefatos `.elf`, `.uf2` e `.ulf` de teste foram removidos, seguidos
de `cargo clean` no workspace e no diretório `firmware/target` legado.

### Revisão do linker RP2350

Uma tentativa de substituir integralmente o `link.x` do `cortex-m-rt` criou um
`rp2350-link.x` privado. O arquivo continha uma diretiva `INSERT AFTER` em
posição sintaticamente inválida e causou falha no `rust-lld`.

Solução adotada:

- removido `rp2350-link.x`;
- restaurado o `link.x` oficial do `cortex-m-rt 0.7.5`;
- mantida somente a seção `.start_block` RP2350 em `memory.x`;
- `FLASH` inicia em `0x10000000` e `.start_block` permanece nos primeiros
  4 KiB examinados pela Boot ROM;
- RAM mantida como região contínua `0x20000000..0x20082000` (520 KiB);
- rastreamento do hash Git corrigido para observar o HEAD e a ref reais do
  repositório, em vez do caminho inexistente `firmware/.git/HEAD`.

### Estado de validação

A compilação, lint, testes completos e geração do UF2 final foram
deliberadamente adiados até o encerramento de todas as alterações desta rodada.

---
