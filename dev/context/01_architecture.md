# OpenHOTAS — Arquitetura & Modelo de Execução

> **LEIA ESTE ARQUIVO PRIMEIRO.**
> Contrato arquitetural estável. Só muda com decisão explícita documentada em log/.

---

## 1. Identidade do Projeto

**OpenHOTAS** é um controlador HOTAS (Hands On Throttle And Stick) open-source
baseado no microcontrolador RP2350, implementado em Rust `no_std` com Embassy 0.10.

### Hardware Alvo

| Componente | Especificação |
|---|---|
| MCU | RP2350 (Raspberry Pi Pico 2) — Cortex-M33 |
| Target Rust | `thumbv8m.main-none-eabihf` |
| Eixos | 3× MT6826S via SPI1 (X, Y, Twist) |
| Botões | 2× MCP23S17 via SPI0 → 32 botões |
| USB | HID Gamepad via embassy-usb, polling 1ms |
| Flash | 2MB interna |
| Paradigma | `no_std`, `no_heap`, async Embassy 0.10 |

### Regra Crítica de Escopo — INVIOLÁVEL

Este firmware gerencia **APENAS o Joystick (Stick)**.

O Throttle é um projeto de hardware 100% independente em outro microcontrolador.

**É EXPRESSAMENTE PROIBIDO** adicionar ao firmware:
- Lógica de throttle
- Eixos extras além dos 3 existentes
- Chaves seletoras de quadrante
- Qualquer lógica que não seja joystick

---

## 2. Modelo de Execução — 4 Tasks

O firmware opera com exatamente 4 tasks assíncronas no executor Embassy.

| Task | Módulo | Dispara | Responsabilidade |
|---|---|---|---|
| `usb_task` | `tasks/hid.rs` | loop | `device.run().await` — mantém stack USB viva |
| `hid_task` | `tasks/hid.rs` | `REPORT_SIGNAL.wait()` | Envia report HID de 10 bytes a ~1ms |
| `input_task` | `tasks/input.rs` | loop livre | Lê 3× MT6826S + 2× MCP23S17 → pipeline → signal |
| `diagnostic_task` | `tasks/diagnostic.rs` | `Timer::after_secs(5)` | Loga RuntimeStats via defmt |

### Restrições Críticas de Execução

**`input_task` é monolítica e indivisível na V1.x.**
Não pode ser fragmentada em `sensor_task` + `axis_task`. O fluxo contínuo
garante latência abaixo de 500µs (`MAX_INPUT_CYCLE_US`).

**`main.rs` é restrito a:**
- Inicialização de hardware (SPI, GPIO, Flash, USB)
- Declaração de buffers `static mut`
- `spawner.spawn(...)` das 4 tasks

Toda lógica operacional reside em `src/tasks/`. Meta: ~120 linhas em `main.rs`.

---

## 3. Estrutura de Módulos

```
src/
├── main.rs                  # Init + spawn. SEM lógica de negócio.
├── constants.rs             # Fonte única de constantes — nunca redefinir localmente
├── spi_bus.rs               # SPI0/SPI1 globais com critical_section
├── sensors/
│   ├── mod.rs               # trait Sensor, enum SensorError
│   ├── mt6826.rs            # Encoder absoluto 15-bit — Burst Read
│   └── mcp23s.rs            # Expansor I/O — 32 botões com debounce
├── calibration/
│   ├── mod.rs
│   ├── data.rs              # CalibrationData — normalização u16 → f32
│   └── cal_store.rs         # Persistência em flash (CALIB_OFFSET)
├── filters/
│   ├── mod.rs
│   ├── max_jump.rs          # Rejeição de spikes (PRIMEIRO filtro)
│   ├── ema.rs               # Suavização exponencial
│   ├── deadzone.rs          # Zona morta simétrica
│   ├── expo.rs              # Curva exponencial
│   └── response_curve.rs    # Pass-through V1.x — stub V2
├── axis/
│   ├── mod.rs               # AxisConfig, AxisOutput
│   └── pipeline.rs          # Orquestra pipeline completo
├── config/
│   ├── mod.rs
│   └── settings.rs          # DeviceConfig — load/save + CRC32
├── storage/
│   ├── mod.rs
│   └── flash.rs             # Primitivas: read, erase, write, crc32
├── usb/
│   ├── mod.rs
│   ├── descriptor.rs        # HID Report Descriptor
│   └── hid_gamepad.rs       # GamepadReport, REPORT_SIGNAL, axis_to_i16
├── tasks/
│   ├── mod.rs
│   ├── input.rs             # input_task
│   ├── hid.rs               # usb_task + hid_task
│   └── diagnostic.rs        # diagnostic_task
└── diagnostics/
    ├── mod.rs
    ├── sensor_health.rs     # SensorStatus
    └── runtime_stats.rs     # Contadores AtomicU32
```

---

## 4. Naming Contract

Obrigatório para compatibilidade com ferramentas futuras de configuração via PC.

| Prefixo | Uso |
|---|---|
| `MAGIC_` | Magic numbers de validação de flash (ex: `MAGIC_DEVICE`, `MAGIC_CAL`) |
| `CONFIG_` | Constantes de configuração (ex: `CONFIG_VERSION`, `CONFIG_OFFSET`) |
| `CALIB_` | Constantes de calibração (ex: `CALIB_OFFSET`) |
| `DEFAULT_` | Valores padrão de tuning — **apenas** dentro de `constants::tuning` |
| `MT6826_` | Constantes do sensor encoder |
| `MCP23S17_` | Constantes do expansor de I/O |

### Regras de Import de Constantes

```rust
// ✅ Correto — filtros usam namespace tuning
use crate::constants::tuning::DEFAULT_EMA_ALPHA;

// ❌ Proibido — nunca importar DEFAULT_* do namespace raiz
use crate::constants::DEFAULT_EMA_ALPHA;

// ❌ Proibido — nunca redefinir constantes localmente
const MY_ALPHA: f32 = 0.3; // em qualquer módulo
```

### Naming de Módulos — Evitar Clippy Inception

Módulos internos não podem ter nome idêntico ao seu diretório raiz.

```
✅ src/calibration/data.rs        → calibration::data::Calibration
❌ src/calibration/calibration.rs → calibration::calibration::Calibration  (ambíguo)
```

---

## 5. Feature Flags Obrigatórias (Cargo.toml)

```toml
[dependencies]
embassy-rp = { version = "0.10", features = [
    "rp235xa",               # Target RP2350 variante A
    "time-driver",           # Timer async
    "critical-section-impl", # Segurança de concorrência em barramentos compartilhados
    "unstable-pac",          # Acesso ao PAC
    "rom-v2-intrinsics",     # Intrínsecas ROM do RP2350
    "imagedef-secure-exe",   # OBRIGATÓRIO — bootloader RP2350 reconhece o binário
] }
```

`imagedef-secure-exe` é mandatório. Sem ela o bootloader do RP2350 não reconhece
o firmware como executável válido.

---

## 6. Riscos Técnicos Aceitos (Single-Core)

Estes padrões são **sound** apenas em ambiente single-core. Se o projeto migrar
para multicore (SMP), precisam ser refatorados.

| Padrão | Onde | Condição de segurança |
|---|---|---|
| `Mutex<CriticalSectionRawMutex, RefCell<Option<...>>>` para SPI | `spi_bus.rs` | Single-core, sem DMA concorrente |
| `static mut` + `critical_section` para Flash | `storage/flash.rs` | Single-core, nunca em ISR |
| `transmute` de lifetimes locais → `'static` | `main.rs` | Inicialização única, single-core |

> Padrões eliminados na V1.2: `static mut` para SPI (→ `Mutex`) e raw pointer
> `*mut Ema` na Deadzone (→ flag booleana). Ver `log/v1_2_build.md`.

---

*OpenHOTAS · Arquitetura V1.2 · Jun/2026*
