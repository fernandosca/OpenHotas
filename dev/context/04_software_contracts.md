# OpenHOTAS — Contratos de Software

> Regras operacionais do firmware. Complementa `dev/context/01_architecture.md`.
> Qualquer desvio é classificado como **BUG DE ARQUITETURA**.
> Última atualização: V1.4 (Jul/2026)

---

## 1. Ciclo de Vida dos Tipos de Sinal

Todo sinal de eixo passa obrigatoriamente por estas 3 formas, nesta ordem:

| Estágio | Tipo | Range | Onde ocorre |
|---|---|---|---|
| **Raw** | `u16` | `0..=32767` | Saída do sensor MT6826S |
| **Normalizado** | `f32` | `[-1.0, 1.0]` | Após calibração — toda computação de filtros |
| **HID Output** | `i16` | `[-32767..=32767]` | Conversão final antes do report USB |

```
MT6826S → u16 → [Calibração] → f32 → [Filtros] → f32 → [axis_to_i16] → i16 → HID
```

**Invariante:** Nenhum filtro opera sobre `u16` ou `i16`. Toda matemática é `f32`.

---

## 2. Pipeline de Sinal — Ordem Absoluta

Cada amostra de eixo percorre `AxisPipeline::process()` nesta ordem **exata e imutável**:

```
1. Calibração      (calibration/data.rs)
2. Center Offset   (axis/pipeline.rs)      ← V1.3: ajuste fino do zero
3. Travel Limits   (axis/pipeline.rs)      ← V1.22: corrige curso físico
4. MaxJump         (filters/max_jump.rs)
5. EMA             (filters/ema.rs)
6. Deadzone        (filters/deadzone.rs)
7. ResponseCurve   (filters/response_curve.rs)  ← piecewise linear
```

### Justificativa da Ordem

| Posição | Filtro | Por quê nesta posição |
|---|---|---|
| 1 | Calibração | Converte u16 → f32. Tudo mais opera em f32. |
| 2 | Center Offset | Ajusta zero **após** calibração, **antes** de qualquer processamento |
| 3 | Travel Limits | **Após** center offset, **antes** dos filtros — corrige curso físico |
| 4 | MaxJump | **Antes** do EMA — spike rejeitado não contamina o histórico do filtro |
| 5 | EMA | Suavização sobre sinal já validado |
| 6 | Deadzone | **Depois** do EMA — zona morta incide sobre sinal estável |
| 7 | ResponseCurve | Curva piecewise linear com 5 pontos de controle |

> ⚠️ Não reordenar. A justificativa de cada posição é funcional, não arbitrária.

### Travel Limits Simétrico

`AxisTravelLimits` usa um único campo público:

```rust
pub travel_limit_pct: u8 // Range válido: 1..=100
```

Após a calibração, o centro mecânico é `0.0`. O limite de curso é simétrico:
o mesmo percentual vale para o lado negativo e para o lado positivo.

Exemplo:

```text
travel_limit_pct = 95
input normalizado ±0.95 → saída HID ±100%
```

Não usar janela `input_min_pct/input_max_pct` do curso total. Esse modelo foi
substituído porque joystick centralizado deve ter curso equivalente para ambos
os lados a partir do centro.

### Regra Especial — Eixo Twist

Quando o sinal entra na Deadzone (`fabsf(input) < threshold`), o EMA é resetado
para evitar drift de retorno mecânico.

A partir da **V1.2**, o mecanismo é 100% safe: `Deadzone::apply()` retorna
`(f32, bool)` onde o `bool` sinaliza a transição de entrada na zona morta.
`AxisPipeline::process()`, como owner de ambos `ema` e `deadzone`, chama
`ema.reset()` diretamente ao receber `true`:

```rust
// Deadzone::apply() — V1.2
pub fn apply(&mut self, input: f32) -> (f32, bool) {
    // retorna (valor, just_entered_deadzone)
}

// AxisPipeline::process() — V1.2
let (dz, reset_ema) = self.deadzone.apply(smt);
if reset_ema {
    self.ema.reset(); // owner chama diretamente, sem unsafe
}
```

`DeviceConfig::default()` define `axes[2].reset_ema_on_dz = true` (índice 2 = Twist).
A flag `reset_ema_on_dz` em `AxisConfig` continua existindo — controla se o
pipeline executa o reset ao receber `true` da Deadzone.

### Clamp Obrigatório

Todo estágio do pipeline deve garantir saída em `[-1.0, 1.0]`.

```rust
// Clamp defensivo no output final de AxisPipeline::process()
AxisOutput {
    value: value.clamp(-1.0, 1.0),
    healthy,
}
```

### Inversão de Eixo

Aplicada no output final, após todos os filtros:

```rust
let value = if self.config.inverted { -out } else { out };
```

---

## 3. Calibração — Contrato

`CalibrationData` define os limites físicos de um eixo no domínio circular de 15 bits:

```rust
pub struct CalibrationData {
    pub center: u16,  // posição central mecânica real
    pub min:    u16,  // mínimo físico atingível
    pub max:    u16,  // máximo físico atingível
}
```

**Default pré-calibração:**
```rust
center: MT6826_ANGLE_CENTER, // 16384
min:    0,
max:    MT6826_ANGLE_MAX,    // 32767
```

### Calibração Circular

O MT6826S é um encoder absoluto circular (0..32767). A calibração usa
`circular_delta()` para calcular a menor distância signed entre `raw` e `center`
no círculo de 15 bits:

```rust
delta = ((raw - center + 16384) mod 32768) - 16384
```

Isso garante que a transição `32767 → 0` (cruzamento do zero físico) seja
contínua para qualquer eixo com curso menor que meia revolução.

**Normalização em `Calibration::apply(raw)`:**
- `delta == 0` → retorna 0.0 (raw == center)
- `delta` mesmo sinal de `min_delta` → lado do min, resultado ∈ `[-1.0, 0.0)`
- `delta` sinal oposto → lado do max, resultado ∈ `(0.0, 1.0]`
- Se calibração inválida (`is_valid` false) → retorna 0.0 (degradado)

### Validez dos Pontos

`is_valid(minimum_span)` exige:
1. `min_delta != 0` (min diferente de center)
2. `max_delta != 0` (max diferente de center)
3. `min` e `max` em lados opostos de center (signum diferentes)
4. `span >= minimum_span` (curso mínimo ≥ 1000 contagens)

Ordem numérica dos pontos brutos **não importa** — o algoritmo detecta
automaticamente a direção do sensor.

### Captura via CDC

`CaptureCalibrationPoint` só pode aceitar uma amostra se o eixo estiver saudável.
Se o bit correspondente em `SENSOR_UNHEALTHY` estiver ativo, o firmware retorna
`ProtocolError::CalibrationError` e não grava o ponto.

Essa regra impede persistir o fallback de centro (`MT6826_ANGLE_CENTER`) como
se fosse uma leitura física válida durante falhas de CRC ou magneto.

---

## 4. Falhas de Sensor — Comportamento

Falhas de CRC ou STATUS de magneto **não travam o sistema**.

```rust
// Em input_task:
let rx = sens_x.read().ok();  // None em caso de erro

// Pipeline recebe centro quando falha:
let out_x = pl_x.process(rx.unwrap_or(MT6826_ANGLE_CENTER), rx.is_some());
//                                                            ↑ healthy = false
```

- O eixo retorna o valor do centro (0.0 normalizado)
- `AxisOutput.healthy = false` sinaliza a falha
- Contadores de erro são incrementados em `runtime_stats`
- O frame HID é enviado normalmente — o host vê o eixo travado no centro

### Falhas do Expansor de Botões

Falha de init ou leitura dos MCP23S17 **não trava o boot nem o HID**.

- O firmware continua operando os eixos
- Botões são reportados como soltos (`0xFFFF_FFFF` antes da inversão)
- `BUTTON_ERRORS` é incrementado
- `BUTTONS_DEGRADED` fica ativo para `GetErrorCounters`

---

## 5. Comunicação entre Tasks

Tasks se comunicam **exclusivamente** via primitivas Embassy:

| Mecanismo | Tipo | Uso |
|---|---|---|
| `REPORT_SIGNAL` | `Signal<CriticalSectionRawMutex, GamepadReport>` (latest-wins, non-blocking) | `input_task` → `hid_task` |
| `CONFIG_SIGNAL` | `Channel<CriticalSectionRawMutex, RuntimeConfig, 1>` | `cdc_task` → `input_task` (latest-wins V1.23) |
| `AtomicU32` | `runtime_stats` globais | `input_task` → `cdc_task` / `diagnostic_task` |

```rust
// Declaração global em hid_gamepad.rs
pub static REPORT_SIGNAL: Signal<CriticalSectionRawMutex, GamepadReport> = Signal::new();

// input_task publica:
REPORT_SIGNAL.signal(GamepadReport { x, y, twist, buttons });

// hid_task consome:
let report = REPORT_SIGNAL.wait().await;
```

> `Signal` descarta valores não consumidos — se `hid_task` estiver ocupada,
> o report anterior é sobrescrito. Comportamento correto para HID em tempo real.

---

## 6. HID Report — Formato

10 bytes por report. Sem Report ID (removido na V1.22 — único report HID).

| Bytes | Conteúdo | Tipo |
|---|---|---|
| [0..2] | Eixo X | `i16` little-endian |
| [2..4] | Eixo Y | `i16` little-endian |
| [4..6] | Eixo Twist (Rz) | `i16` little-endian |
| [6..10] | Botões B0..B31 | `u32` little-endian |

```rust
pub fn axis_to_i16(v: f32) -> i16 {
    (v.clamp(-1.0, 1.0) * HID_AXIS_MAX as f32) as i16
    //                     ↑ 32767
}
```

---

## 7. Flash — Regras de Uso

### Layout de Memória (V1.4)

```
Offset físico          Conteúdo
─────────────────────────────────────────────
0x00000000             [ código + dados ]
      ...
0x001FE000             STORED_V2_SLOT_B (setor de backup, 4KB)
                       [ MAGIC "OCFG"(4) | generation(4) | storage_version(1) |
                         proto_major(1) | proto_minor(1) | payload_len(2) |
                         postcard(DeviceConfig) | CRC32(4) ]
0x001FF000             STORED_V2_SLOT_A (setor primário, 4KB)
                       [ mesmo layout ]
0x00200000             FLASH_SIZE = 2MB
```

> Double-buffer com geração para power-fail safety. Boot lê ambos os slots,
> usa o de maior geração. Save escreve no slot inativo com geração + 1.
> Ver detalhes em `§9 StoredConfigV2`.

> ⚠️ Offsets são **físicos** (relativos ao início da flash, base `0x00`).
> **Nunca** usar endereços XIP absolutos (`0x10000000`) nas funções de escrita.
> Para leitura via ponteiro XIP: `let ptr = (0x10000000u32 + offset) as *const u8;`

### Invariantes de Escrita — INVIOLÁVEIS

1. **Erase obrigatório antes de write:** `blocking_erase` de setor inteiro (4096 bytes) antes de qualquer `blocking_write`. Escrever sem apagar antes corrompe bits.

2. **Nunca em ISR:** `blocking_write` congela o barramento XIP durante a operação. Qualquer ISR que acesse a flash causa crash imediato.

3. **Alinhamento de setor:** O offset de erase/write deve ser múltiplo de `SECTOR_SIZE` (4096).

```rust
// Sequência obrigatória para salvar dados (StoredConfigV2 double-buffer):
let target = if active_slot == STORED_V2_SLOT_A { STORED_V2_SLOT_B } else { STORED_V2_SLOT_A };
flash::erase_sector(target)?;       // 1. Apagar slot inativo
flash::write_flash(target, &buf)?;  // 2. Escrever no slot inativo
```

### Validação de Leitura — Dupla Checagem

Dados lidos da flash são **sempre** validados antes de usar:

```rust
// 1. Magic number correto? ("OCFG")
if magic != MAGIC_V2 { return DeviceConfig::default(); }

// 2. Storage version compatível?
if storage_version != STORAGE_VERSION { return DeviceConfig::default(); }

// 3. Protocol major compatível?
if proto_major != PROTOCOL_VERSION_MAJOR { return DeviceConfig::default(); }

// 4. CRC32 bate?
if stored_crc != computed_crc { return DeviceConfig::default(); }
```

Se qualquer verificação falhar → usar `Default::default()`. Nunca usar dados corrompidos.

---

## 8. USB — Estrutura de Buffers

Os buffers do `UsbBuilder` usam `StaticCell` em `main.rs` — exigência de
lifetime `'static` do `embassy-usb 0.5`, sem `unsafe`.

```rust
// Declaração global em main.rs (V1.3+)
use static_cell::StaticCell;

static DD: StaticCell<[u8; 256]> = StaticCell::new(); // device descriptor
static CD: StaticCell<[u8; 256]> = StaticCell::new(); // config descriptor
static BD: StaticCell<[u8; 256]> = StaticCell::new(); // bos descriptor
static CB: StaticCell<[u8; 64]>  = StaticCell::new(); // control buf
static HS: StaticCell<embassy_usb::class::hid::State<'static>> = StaticCell::new();
/// CDC state para protocolo binário (V1.22).
static CDC_STATE: StaticCell<embassy_usb::class::cdc_acm::State<'static>> = StaticCell::new();
static SERIAL_STR: StaticCell<[u8; 18]> = StaticCell::new();
```

Inicialização em `main()`:

```rust
let hs = HS.init(embassy_usb::class::hid::State::new());
let dd = DD.init([0u8; 256]);
// ...
let mut builder = Builder::new(driver, usb_cfg, dd, cd, bd, cb);
```

Estes buffers **nunca** são alocados em stack ou heap.

---

## 8.1. Serial USB — Formato do Contrato

O serial number USB é `"OH"` seguido de 16 caracteres hex uppercase
(`"OH{:016X}"`), totalizando 18 bytes.

| Componente | Tamanho | Conteúdo |
|---|---|---|
| Prefixo | 2 bytes | `"OH"` (ASCII fixo) |
| UID | 16 hex chars | 64-bit chip ID do RP2350 (8 bytes, big-endian, 2 hex por byte) |

Exemplo: `OH0A1B2C3D4E5F6071`

**Fonte:** chip ID de OTP, lido via `embassy_rp::otp::get_chipid()` (rows
0x0–0x3). Cada RP2350 tem um chip ID único gravado em fábrica.

> ⚠️ Não usar os endereços SYSINFO (`0x00010040`/`0x00010044`) — base
> incorreta; a leitura "funcionava" por acaso (lixo da ROM sem crash). A fonte
> oficial é OTP.

**Fallback de degradação:**
Se a OTP estiver ilegível (falha teórica), o firmware usa o serial fixo
`OH0000000000000000` e emite um `defmt::warn!`. O boot não trava, mas todos os
sticks nessa condição compartilham o mesmo serial.

**Contrato para ferramentas (CLI/GUI):**
- O serial **não** é usado para discovery — o CLI e GUI identificam o dispositivo
  via `Request::GetInfo` → `DeviceInfo.protocol_major`.
- O serial serve apenas para **uniqueness na enumeração USB** — dois OpenHOTAS
  no mesmo host não colidem.
- Formato estável: se mudar, é breaking change no USB descriptor.
- O serial `OH0000000000000000` indica OTP ilegível — o configurador deve
  alertar o usuário ao encontrá-lo.

---

## 9. StoredConfigV2 — Estrutura de Persistência (V1.4)

A persistência usa double-buffer com geração para power-fail safety. Dois slots
em flash alternam a cada gravação — o slot ativo nunca é alterado durante escrita.

```text
Offset  Tamanho  Campo
────────────────────────────────────
0       4        MAGIC = "OCFG"
4       4        GENERATION (u32 LE, incrementa a cada save)
8       1        STORAGE_VERSION = 2
9       1        PROTOCOL_MAJOR
10      1        PROTOCOL_MINOR
11      2        PAYLOAD_LEN u16 big-endian
13      N        PAYLOAD = postcard(DeviceConfig)
13+N    4        CRC32 (cobre bytes 4 até 13+N-1)
```

### Slots

| Slot | Offset | Constante |
|------|--------|-----------|
| A | 0x1FF000 | `STORED_V2_SLOT_A` = `FLASH_SIZE - SECTOR_SIZE` |
| B | 0x1FE000 | `STORED_V2_SLOT_B` = `FLASH_SIZE - 2 * SECTOR_SIZE` |

### Fluxo de Write (`save_config`)

1. Lê ambos os slots, valida magic/version/CRC
2. Identifica slot ativo (maior geração)
3. Alvo = slot **inativo**
4. Serializa payload em buffer RAM
5. **Erase** do slot inativo
6. **Write** do buffer (MAGIC + geração+1 + header + payload + CRC32)
7. Se qualquer etapa falhar → incrementa `FLASH_ERRORS`, retorna erro

### Power-Fail Safety

- Se power falha durante erase do slot inativo → slot ativo intacto
- Se power falha durante write → CRC rejeita slot parcial no próximo boot
- **Nunca há janela onde ambos os slots são inválidos**

### Fluxo de Read (`load_config`)

1. Lê ambos os slots via `read_slot` (valida magic, version, protocol major, CRC32)
2. Ambos válidos → usa o de maior geração
3. Apenas um válido → usa esse
4. Nenhum válido → retorna `DeviceConfig::default()`

### Invariantes

- `SetConfig` **não** grava flash
- `LoadDefaults` **não** grava flash
- `SaveConfig` grava flash
- `FactoryReset` grava flash (defaults) + reboot

---

## 10. O que NÃO Fazer — Regras Absolutas

```
❌ Dividir input_task em sensor_task + axis_task
❌ Adicionar lógica de throttle
❌ Redefinir constantes localmente nos módulos
❌ Importar DEFAULT_* do namespace raiz (usar constants::tuning::*)
❌ Chamar blocking_write de dentro de ISR
❌ Escrever na flash sem apagar o setor antes
❌ Usar endereços XIP absolutos nos offsets de flash
❌ Criar módulo com nome igual ao seu diretório pai
❌ Fragmentar main.rs com lógica de negócio
❌ Alterar a ordem do pipeline de sinal
```

---

*OpenHOTAS · Contratos de Software V1.4 · Jul/2026*
