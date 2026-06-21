# OpenHOTAS — Contratos de Software

> Regras operacionais do firmware. Complementa `dev/context/01_architecture.md`.
> Qualquer desvio é classificado como **BUG DE ARQUITETURA**.

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
1. Calibração     (calibration/data.rs)
2. MaxJump        (filters/max_jump.rs)
3. EMA            (filters/ema.rs)
4. Deadzone       (filters/deadzone.rs)
5. Expo           (filters/expo.rs)
6. ResponseCurve  (filters/response_curve.rs)   ← pass-through V1.x
```

### Justificativa da Ordem

| Posição | Filtro | Por quê nesta posição |
|---|---|---|
| 1 | Calibração | Converte u16 → f32. Tudo mais opera em f32. |
| 2 | MaxJump | **Antes** do EMA — spike rejeitado não contamina o histórico do filtro |
| 3 | EMA | Suavização sobre sinal já validado |
| 4 | Deadzone | **Depois** do EMA — zona morta incide sobre sinal estável |
| 5 | Expo | Curva de resposta sobre saída já processada |
| 6 | ResponseCurve | Pass-through V1.x — posição reservada para V2 |

> ⚠️ Não reordenar. A justificativa de cada posição é funcional, não arbitrária.

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

`CalibrationData` define os limites físicos de um eixo:

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

**Normalização em `Calibration::apply(raw)`:**
- `raw <= center` → range = `center - min`, resultado ∈ `[-1.0, 0.0]`
- `raw > center`  → range = `max - center`, resultado ∈ `[0.0, 1.0]`
- Se range == 0.0 → retorna 0.0 (proteção contra divisão por zero)

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

---

## 5. Comunicação entre Tasks

Tasks se comunicam **exclusivamente** via primitivas Embassy:

| Mecanismo | Tipo | Uso |
|---|---|---|
| `REPORT_SIGNAL` | `Signal<CriticalSectionRawMutex, GamepadReport>` | `input_task` → `hid_task` |
| `AtomicU32` | `runtime_stats` globais | `input_task` → `diagnostic_task` |

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

10 bytes por report. Report ID = `0x01`.

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

### Layout de Memória

```
Offset físico          Conteúdo
─────────────────────────────────────────────
0x00000000             [ código + dados ]
      ...
0x001FE000             CALIB_OFFSET (penúltimo setor, 4KB)
                       [ MAGIC_CAL(4) | centers/mins/maxs(18) | CRC32(4) ]
0x001FF000             CONFIG_OFFSET (último setor, 4KB)
                       [ MAGIC_DEVICE(4) | version(1) | profile(1) | axes(72) | CRC32(4) ]
0x00200000             FLASH_SIZE = 2MB
```

> ⚠️ Offsets são **físicos** (relativos ao início da flash, base `0x00`).
> **Nunca** usar endereços XIP absolutos (`0x10000000`) nas funções de escrita.
> Para leitura via ponteiro XIP: `let ptr = (0x10000000u32 + offset) as *const u8;`

### Invariantes de Escrita — INVIOLÁVEIS

1. **Erase obrigatório antes de write:** `blocking_erase` de setor inteiro (4096 bytes) antes de qualquer `blocking_write`. Escrever sem apagar antes corrompe bits.

2. **Nunca em ISR:** `blocking_write` congela o barramento XIP durante a operação. Qualquer ISR que acesse a flash causa crash imediato.

3. **Alinhamento de setor:** O offset de erase/write deve ser múltiplo de `SECTOR_SIZE` (4096).

```rust
// Sequência obrigatória para salvar dados:
flash::erase_sector(CONFIG_OFFSET)?;   // 1. Apagar
flash::write_flash(CONFIG_OFFSET, &buf)?; // 2. Escrever
```

### Validação de Leitura — Dupla Checagem

Dados lidos da flash são **sempre** validados antes de usar:

```rust
// 1. Magic number correto?
if magic != MAGIC_DEVICE { return Self::default(); }

// 2. Versão compatível?
if version != CONFIG_VERSION { return Self::default(); }

// 3. CRC32 bate?
if stored_crc != computed_crc { return Self::default(); }
```

Se qualquer verificação falhar → usar `Default::default()`. Nunca usar dados corrompidos.

---

## 8. USB — Estrutura de Buffers

Os buffers do `UsbBuilder` são `static mut` em `main.rs` — exigência de
lifetime `'static` do `embassy-usb 0.5`.

```rust
// Declaração global em main.rs
static mut DD: [u8; 256] = [0u8; 256]; // device descriptor
static mut CD: [u8; 256] = [0u8; 256]; // config descriptor
static mut BD: [u8; 256] = [0u8; 256]; // bos descriptor
static mut CB: [u8; 64]  = [0u8; 64];  // control buf
static mut HS: Option<embassy_usb::class::hid::State<'static>> = None;
```

Estes buffers **nunca** são alocados em stack ou heap.

---

## 9. DeviceConfig — Estrutura de Persistência

```rust
pub struct DeviceConfig {
    pub magic:          u32,          // MAGIC_DEVICE = 0x484F5441 ("HOTA")
    pub version:        u8,           // CONFIG_VERSION = 1
    pub active_profile: u8,           // reservado — V2
    pub axes:           [AxisConfig; 3], // índices: AXIS_X=0, AXIS_Y=1, AXIS_TWIST=2
}
```

```rust
pub struct AxisConfig {
    pub ema_alpha:      f32,   // DEFAULT_EMA_ALPHA = 0.3
    pub deadzone:       f32,   // DEFAULT_DEADZONE  = 0.02
    pub max_jump:       f32,   // DEFAULT_MAX_JUMP  = 0.15
    pub expo:           f32,   // DEFAULT_EXPO      = 0.0
    pub inverted:       bool,
    pub reset_ema_on_dz:bool,  // true apenas em axes[2] (Twist)
}
```

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

*OpenHOTAS · Contratos de Software V1.2 · Jun/2026*
