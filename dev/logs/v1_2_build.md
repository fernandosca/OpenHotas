# OpenHOTAS — Build Log V1.2

> **Status final:** Compilou sem erros, sem warnings, Clippy limpo (zero warnings).
> **Data:** Jun/2026
> **Toolchain:** Rust 1.96.0 stable, `thumbv8m.main-none-eabihf`
> **Destaques:** Zero blocos `unsafe` no caminho crítico. `#![allow(static_mut_refs)]` removido.

---

## Contexto da Versão

A V1.1 compilou limpa, mas dois padrões dependiam de invariantes invisíveis ao compilador:

1. `spi_bus.rs`: `static mut` + `critical_section` manual — exigia `#![allow(static_mut_refs)]` e blocos `unsafe` explícitos
2. `filters/deadzone.rs`: raw pointer `*mut Ema` para resetar o EMA do eixo Twist — contornava o borrow checker via `unsafe`

Ambos eram *sound* em single-core (sem UB no contexto do RP2350), mas dependiam de garantias que o compilador não podia verificar. A V1.2 substitui os dois por equivalentes 100% safe sem perda de performance nem mudança de comportamento.

**Nenhuma lógica de negócio foi alterada. Nenhuma dependência foi adicionada.**

---

## Arquivos Alterados

| # | Arquivo | Ação | Mudança principal |
|---|---|---|---|
| 1 | `src/spi_bus.rs` | Reescrito | `static mut` → `Mutex<CriticalSectionRawMutex, RefCell<Option<...>>>` |
| 2 | `src/filters/deadzone.rs` | Reescrito | Raw pointer eliminado; `apply()` retorna `(f32, bool)` |
| 3 | `src/axis/pipeline.rs` | Editado | `process()` consome flag booleana e chama `ema.reset()` diretamente |
| 4 | `src/main.rs` | Editado | `#![allow(static_mut_refs)]` removido do crate root |

**Total:** 4 arquivos (2 reescritos, 2 editados). Zero dependências adicionadas.

---

## Refatoração 1 — Deadzone sem Raw Pointer

### Problema (V1.1)

`Deadzone::apply()` usava `Option<*mut Ema>` para resetar o EMA do Twist.
`AxisPipeline` era o owner legítimo de ambos, mas o borrow checker não deixava
dois `&mut` simultâneos, então o raw pointer era o workaround:

```rust
// V1.1 — unsafe no caminho crítico
pub struct Deadzone {
    threshold: f32,
    pub(crate) ema_ref: Option<*mut Ema>, // raw pointer
    in_zone: bool,
}

pub fn apply(&mut self, input: f32) -> f32 {
    if fabsf(input) < self.threshold {
        if !self.in_zone {
            if let Some(ema) = self.ema_ref {
                unsafe { (*ema).reset(); } // unsafe
            }
        }
        // ...
    }
}
```

### Solução (V1.2)

`apply()` retorna `(f32, bool)` — o `bool` sinaliza a transição de entrada
na zona morta. O pipeline, como owner de `ema` e `deadzone`, consome a flag
e chama `reset()` diretamente:

```rust
// V1.2 — 100% safe
pub struct Deadzone {
    threshold: f32,
    in_zone: bool,  // ema_ref removido
}

pub fn apply(&mut self, input: f32) -> (f32, bool) {
    let input = input.clamp(-1.0, 1.0);
    let mut just_entered = false;

    if fabsf(input) < self.threshold {
        if !self.in_zone {
            just_entered = true;
        }
        self.in_zone = true;
        return (0.0, just_entered);
    }

    self.in_zone = false;
    let sign = if input >= 0.0 { 1.0 } else { -1.0 };
    (sign * (fabsf(input) - self.threshold) / (1.0 - self.threshold), false)
}
```

```rust
// AxisPipeline::process() — V1.2
let (dz, reset_ema) = self.deadzone.apply(smt);
if reset_ema {
    self.ema.reset(); // owner chama diretamente, sem unsafe
}
```

**Impacto:** único `*mut` do projeto removido. `with_ema_reset()` e seu
`#[allow(dead_code)]` eliminados. Comportamento idêntico à V1.1.

---

## Refatoração 2 — SPI Bus Seguro

### Problema (V1.1)

`spi_bus.rs` usava `static mut` protegido por `critical_section::with`.
Sound em single-core (mesmas instruções `CPSID`/`CPSIE` de qualquer mutex
Cortex-M), mas o compilador não podia verificar a invariante de acesso.
O lint `static_mut_refs` sinalizava isso e precisava ser suprimido:

```rust
// V1.1 — sound mas invisível ao compilador
static mut SPI0_BUS: Option<Spi0Bus> = None;

pub fn with_spi0<R>(f: impl FnOnce(&mut Spi0Bus) -> R) -> R {
    critical_section::with(|_| {
        #[allow(static_mut_refs)]
        unsafe {
            f(SPI0_BUS.as_mut().expect("SPI0 not initialized"))
        }
    })
}
```

### Solução (V1.2)

`embassy_sync::blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<Option<...>>>`.
O `Mutex` com `CriticalSectionRawMutex` emite as mesmas instruções de hardware
— overhead zero. O `RefCell` fornece mutação interior verificada em runtime.
O compilador rastreia o acesso estático corretamente:

```rust
// V1.2 — mesma performance, verificado pelo compilador
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use core::cell::RefCell;

pub static SPI0_BUS: Mutex<CriticalSectionRawMutex, RefCell<Option<Spi0Bus>>> =
    Mutex::new(RefCell::new(None));

pub fn init_spi0(spi: Spi0Bus) {
    SPI0_BUS.lock(|state| { *state.borrow_mut() = Some(spi); });
}

pub fn with_spi0<R>(f: impl FnOnce(&mut Spi0Bus) -> R) -> R {
    SPI0_BUS.lock(|state| {
        let mut bus_ref = state.borrow_mut();
        f(bus_ref.as_mut().expect("SPI0 not initialized"))
    })
}
```

**Impacto:**
- `static mut` de SPI removidos
- `#![allow(static_mut_refs)]` removido do `main.rs`
- Zero `unsafe` restantes no caminho crítico
- Assinaturas de `with_spi0` / `with_spi1` inalteradas para os callers

---

## Decisões Arquiteturais Mantidas da V1.1

| # | Decisão | Status |
|---|---|---|
| 1 | SPI via `embassy_sync::Mutex` | Migrado para este padrão nesta versão |
| 2 | MCP23S17 com CS compartilhado | Mantido — diferenciação via opcode |
| 3 | USB buffers como `static mut` | Mantido — exigência do embassy-usb 0.5 |
| 4 | Flash via `static mut` + `critical_section` | Mantido — candidato à refatoração em V3 |
| 5 | RuntimeStats com `AtomicU32` | Mantido |
| 6 | `input_task` unificada | Mantido — latência abaixo de 500µs |

---

## Riscos Técnicos Conhecidos

| # | Risco | Origem | Status |
|---|---|---|---|
| 1 | SPI via `Mutex` — revisar se SMP for adotado | V1.1 | Aceito |
| 2 | Flash via `static mut` — nunca chamar de ISR | V1.1 | Aceito — candidato a refatoração em V3 |
| 3 | **Burst Read MT6826S não testado em hardware físico** | V1.1 | **Pendente — ação obrigatória antes de V2** |
| 4 | `install_core0_stack_guard()` não implementada | V1.1 | API não localizada no embassy-rp 0.10 |

> ⚠️ O risco #3 é o mais relevante em aberto. Nenhuma versão (V1.0, V1.1, V1.2)
> foi validada em hardware físico. Executar checklist em `02_hardware_specs.md §5`
> antes de qualquer feature nova em V2.

---

## Stubs para V2 (dead_code intencional)

| Módulo | Item | Feature alvo |
|---|---|---|
| `calibration/data.rs` | `Calibration::start/feed/finish` | Calibração runtime |
| `calibration/cal_store.rs` | `save()` | Calibração runtime |
| `config/settings.rs` | `DeviceConfig::save()`, `active_profile` | Configurador PC / Múltiplos perfis |
| `filters/ema.rs` | `set_alpha()` | Configurador PC |
| `filters/max_jump.rs` | `set_threshold()` | Configurador PC |
| `filters/deadzone.rs` | `set_threshold()` | Configurador PC |
| `filters/expo.rs` | `set_factor()` | Configurador PC |
| `filters/response_curve.rs` | interface completa | ResponseCurve customizável |
| `axis/pipeline.rs` | `update_config()` | Configurador PC |
| `usb/descriptor.rs` | `REPORT_ID_CONFIG = 0x02` | Configurador PC |

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

# Tamanho detalhado
cargo size --release --target thumbv8m.main-none-eabihf -- -A
```

---

*OpenHOTAS · Build Log V1.2 · Jun/2026*
