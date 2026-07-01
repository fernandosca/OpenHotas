# OpenHOTAS — Diretrizes de Código para Agentes de IA

> Baseado em: Akita, "Clean Code pra Agentes de IA" (abr/2026) e
> "Boas práticas de projetos de código aberto com LLM" (mai/2026).
> Adaptado ao contexto Rust `no_std` embedded do OpenHOTAS.
>
> **Este arquivo é lido pelo agente antes de qualquer edição de código.**
> Cada regra aqui é obrigatória, não sugestão.

---

## Por que estas regras existem

Agentes de IA têm restrições técnicas reais que impactam qualidade de output:

- **Truncamento de arquivo:** Claude Code lê ~2000 linhas por vez. Arquivo maior = leitura fragmentada = modelo mental incompleto.
- **Contexto finito e degradante:** quanto mais coisa na janela, pior a precisão de detalhe.
- **Grep é a API de navegação primária:** o agente usa `rg`/`grep` constantemente. Nomes únicos e específicos são a interface de busca.
- **Tool calls custam tokens:** arquivo curto, output de teste pequeno, log enxuto — tudo mantém o agente produtivo.

Código limpo não é estética aqui. É restrição técnica.

---

## 1. Tamanho de Funções e Arquivos

```
Funções:   4–30 linhas. Dividir se ultrapassar.
Arquivos:  abaixo de 300 linhas. Idealmente 150–200.
           Dividir por responsabilidade se crescer além disso.
```

No contexto Rust embedded, funções longas quase sempre indicam que lógica
de driver, pipeline e decisão estão misturadas. Separar é correto.

**Exceção aceita:** `main.rs` pode ultrapassar se necessário para o setup
de hardware — mas toda lógica operacional sai para `src/tasks/`.

---

## 2. Single Responsibility Principle

Cada módulo faz uma coisa e tem uma razão para mudar.

```
✅ mt6826.rs       — leitura de sensor MT6826S (só isso)
✅ deadzone.rs     — filtro de zona morta (só isso)
✅ pipeline.rs     — orquestração do pipeline de eixo (só isso)
❌ sensor_pipeline.rs — lê sensor E filtra E serializa
```

Responsabilidades misturadas obrigam o agente a carregar contexto extra
para qualquer mudança simples. Uma mudança no sensor não deve tocar o filtro.

---

## 3. Nomes: Específicos e Únicos (Grepáveis)

Regra prática: **grep pelo nome deve retornar menos de 5 matches no repositório.**

```rust
// ❌ Genérico — grep retorna dezenas de matches
fn process(data: u16) -> f32 { ... }
fn handle(input: f32) -> f32 { ... }

// ✅ Específico — nomes reais do projeto
fn compute_crc8(data: &[u8]) -> u8 { ... }        // único no repo
fn check_magnet(status: u8) -> bool { ... }        // único no repo
fn normalize_angle(raw_angle_15bit: u16) -> f32 { ... }
fn record_cycle_duration(elapsed_us: u32) { ... }
```

Evitar soltos como nome de função ou variável: `data`, `value`, `input`,
`output`, `handler`, `manager`, `process`, `config`. Sempre qualificar
com contexto do domínio (`angle`, `sensor`, `axis`, `calibration`).

---

## 4. Comentários: Proveniência e Decisão, não Tradução

O agente lê código Rust fluentemente. Ele **não sabe**:
- Por que esta abordagem e não a óbvia
- Qual constraint do datasheet força esta ordem ou valor
- Qual workaround existe por limitação do `embassy-rp`
- Qual versão do datasheet corrigiu qual bug

Esses são os comentários que devem existir.

```rust
// ✅ Proveniência — explica decisão não-óbvia
// Burst Read (§8.6.8 datasheet Rev.1.1): CS falling edge faz latch atômico
// dos 4 registradores. Single Byte Read exigiria 4 transações separadas
// e não garantiria consistência de snapshot.
spi.blocking_transfer_in_place(&mut buf)?;

// ✅ Workaround de limitação da HAL documentado
// embassy_rp 0.10 não exporta MODE_3 como constante.
// Configurar CPOL + CPHA manualmente. Pode mudar em versões futuras.
spi1_cfg.polarity = Polarity::IdleHigh;
spi1_cfg.phase = Phase::CaptureOnSecondTransition;

// ✅ Correção de datasheet documentada — evita regressão
// V1.0 usava (status & 0x06) == 0x02 — ERRADO.
// Bit[1]=1 = campo fraco (warning ativo), não OK.
// Campo OK = ambos os bits zerados. Fonte: §8.6.7 Rev.1.1.
(status & MT6826_MAGNET_OK_MASK) == 0x00

// ❌ Tradução — o agente já sabe isso
// Seta CS em low
self.cs.set_low();

// ❌ Óbvio
// Retorna o ângulo
Ok(angle.min(MT6826_ANGLE_MAX))
```

**Regra sobre comentários do agente:** não remover comentários que o agente
escreveu em refactors. Eles carregam contexto de decisão da sessão anterior.
O único comentário do agente que vale remover é o que descreve o óbvio.

---

## 5. Tipos Explícitos — Sem Inferência Ambígua

Rust já força tipos em assinaturas públicas, mas em contexto embedded
as unidades físicas importam e devem ser visíveis:

```rust
// ❌ O que é u16 aqui? Ângulo raw? Microsegundos? Contagem?
fn process(raw: u16) -> f32 { ... }

// ✅ Contexto na assinatura — padrão do projeto
fn apply(&self, raw: u16) -> f32 { ... }          // Calibration::apply
fn record_cycle(us: u32) { ... }                   // runtime_stats
```

Para structs de configuração, sempre documentar a unidade e o range válido
via doc comment `///`:

```rust
pub struct AxisConfig {
    /// Alpha do filtro EMA. Range válido: (0.0, 1.0].
    /// Valores próximos de 1.0 = sem filtragem. Default: 0.3.
    pub ema_alpha: f32,
    /// Zona morta como fração de [-1.0, 1.0].
    /// Ex: 0.02 = 2% de cada lado do centro. Default: 0.02.
    pub deadzone: f32,
    /// Jump máximo entre amostras consecutivas.
    /// Spikes acima deste valor são descartados. Default: 0.15.
    pub max_jump: f32,
}
```

---

## 6. DRY — Sem Duplicação de Lógica

Duplicação em embedded é especialmente perigosa: uma correção de protocolo
aplicada em um lugar e esquecida no outro é um bug silencioso.

```rust
// ❌ CRC calculado inline em dois drivers diferentes
// Se o polinômio mudar, um dos dois vai ficar errado

// ✅ Função central em mt6826.rs referenciada pelo próprio driver
fn compute_crc8(data: &[u8]) -> u8 { ... }
```

Constantes repetidas são o caso mais comum: **nunca redefinir em módulo
local o que já está em `constants.rs`**.

---

## 7. Validação — Hardware é a Fonte de Verdade

Este projeto é um binário embedded puro. Não há harness de teste no host —
o ciclo de validação real é: **build → flash → observar output**.

### Canais de observação disponíveis

| Canal | Como usar | Quando disponível |
|---|---|---|
| `defmt` via probe-rs | `cargo run --release` — output no terminal | Com probe físico conectado |
| CDC Serial | porta COM USB, terminal a 115200 | Após V1.21 implementada |
| Hardware direto | joystick aparece no OS, mover eixos | Sempre após flash |

### Checklist de validação após flash

Executar ao flashar em hardware novo ou após mudança em driver/pipeline:

```
[ ] Joystick aparece no OS como "OpenHOTAS Gamepad"
[ ] Eixo X varia entre 0 e ~32767 ao girar 360°
[ ] Eixo Y varia entre 0 e ~32767
[ ] Eixo Twist varia entre 0 e ~32767
[ ] Nenhum CrcError no output defmt em operação normal
[ ] check_magnet() retorna true com magneto posicionado
[ ] Centro ~16384 na posição mecânica central
[ ] Botões respondem — estado muda no gamepad do OS
[ ] Ciclo dentro de 500µs (diagnostic_task a cada 5s)
```

### Gate de qualidade antes de flashar

O agente executa **todos** estes passos antes de flashar. Falha em qualquer
um = não avançar.

```sh
# 1. Testes de lógica pura — rápido, host, falha cedo
cargo test -p openhotas-filters

# 2. Dependências transitivas — confirmar zero deps embedded
cargo tree -p openhotas-filters --target x86_64-unknown-linux-gnu

# 3. Qualidade de código — clippy e formatação
cargo fmt --check
cargo clippy -p openhotas-filters -- -D warnings
cargo clippy -p openhotas-protocol -- -D warnings

# 4. Build cross do firmware — mais lento, roda por último
cargo build --release
cargo clippy --target thumbv8m.main-none-eabihf -- -D warnings
cd firmware && cargo fmt --check
```

**Regras:**
- `openhotas-filters` é crate library com testes unitários no host
- `cargo tree` deve mostrar **apenas `libm`** — se `embassy-*` ou `cortex-m`
  aparecerem, parar e investigar (extração incorreta)
- `-D warnings` em clippy trata warnings como erros
- Lógica incorreta de filtros é detectada nos testes do crate; lógica de
  hardware é detectada em runtime via defmt ou CDC

---

## 8. `no_std` — Restrições de API que o Agente Esquece

Este projeto é `#![no_std]` e `#![no_heap]`. O agente vai naturalmente
usar APIs de `std` que não existem aqui. Regras específicas:

### Matemática float — usar `libm`, não métodos de `f32`

```rust
// ❌ f32::abs() não existe em no_std sem feature flag
let x = input.abs();

// ✅ libm::fabsf() — já é dependência do projeto
use libm::fabsf;
let x = fabsf(input);

// Outros que aparecem no projeto:
// libm::sqrtf(), libm::floorf(), libm::ceilf()
```

### Logging — usar `defmt`, não `println!`

```rust
// ❌ println! não existe em no_std
println!("ciclo: {}us", elapsed);

// ✅ defmt — já configurado no projeto
defmt::info!("ciclo: {}us", elapsed);
defmt::warn!("spike rejeitado: delta={}", delta);
defmt::error!("CRC mismatch: expected={:#04x} got={:#04x}", expected, buf[5]);
```

### Tempo — usar `embassy_time`, não `std::time`

```rust
// ❌ std::time não existe em no_std
use std::time::Instant;

// ✅ embassy_time — já é dependência do projeto
use embassy_time::Instant;
let start = Instant::now();
let elapsed_us = start.elapsed().as_micros() as u32;
```

### Coleções — sem `Vec`, `HashMap`, `String`

Não existem sem `alloc`. Usar arrays de tamanho fixo, `heapless`, ou
primitivas de tamanho estático. O projeto não usa coleções dinâmicas.

---

## 9. `unsafe` — Regras de Uso

O projeto tem `unsafe` apenas em `main.rs` (transmute de lifetimes para
`'static` nos periféricos). Este é o único `unsafe` aceito.

```rust
// ✅ Padrão aceito — único local, inicialização única, documentado
// Transmute necessário para converter lifetime local do periférico em 'static.
// Sound: inicialização única em main(), single-core, periférico vive
// pelo resto da execução do programa.
let sens_x: Mt6826<'static> =
    unsafe { core::mem::transmute(Mt6826::new(Output::new(p.PIN_10, Level::High))) };
```

**O agente NÃO deve:**
- "Corrigir" os `transmute` existentes — eles são intencionais e necessários
- Adicionar novos `unsafe` fora de `main.rs` sem aprovação explícita
- Usar `unsafe` para contornar erros de borrow checker em filtros ou pipeline
  (a solução correta é o padrão de flag booleana — ver `deadzone.rs` V1.2)

---

## 10. Erros com Contexto

Em embedded, erros silenciosos ou vagos são difíceis de debugar sem probe.
Seguir o padrão já estabelecido no projeto:

```rust
// ❌ Vago — não ajuda o agente nem o desenvolvedor
return Err(SensorError::SpiError);

// ✅ Log antes de retornar o erro — padrão do projeto
defmt::error!("MT6826 CRC mismatch: expected={:#04x} got={:#04x}", expected, buf[5]);
self.error_count = self.error_count.saturating_add(1);
self.last_healthy = false;
return Err(SensorError::CrcError);

// ✅ expect com mensagem acionável para invariantes de inicialização
SPI0_BUS.lock(|s| s.borrow_mut()
    .as_mut()
    .expect("SPI0 not initialized — call init_spi0() before use"))
```

---

## 11. Estrutura de Diretório Previsível

O agente navega o repositório por convenção. Se `src/sensors/mt6826.rs`
existe, o agente antecipa que há `src/sensors/mcp23s.rs` e
`src/sensors/mod.rs`. Qualquer desvio custa tool calls extras de exploração.

**Convenções do OpenHOTAS (não alterar sem atualizar `dev/context/01_architecture.md`):**

```
src/sensors/     — drivers de hardware (implementam trait Sensor)
src/filters/     — re-exports de openhotas-filters (lógica pura)
src/calibration/ — re-exports de openhotas-filters (CalibrationData)
src/tasks/       — tasks Embassy (sem lógica de negócio inline)
src/usb/         — stack HID, descritores, GamepadReport
src/storage/     — primitivas de flash (erase, write, read, crc32)
src/diagnostics/ — telemetria AtomicU32 e SensorStatus
src/axis/        — AxisConfig, AxisOutput, AxisPipeline
src/config/      — DeviceConfig (load/save/validate)

crates/openhotas-filters/ — lógica pura (filtros, calibração, crc32)
crates/openhotas-protocol/ — tipos compartilhados firmware ↔ PC
```

---

## 12. Formatação: Deixar o Toolchain Decidir

```sh
cargo fmt                                         # formatar
cargo clippy --target thumbv8m.main-none-eabihf   # lint
```

Não discutir estilo além disso. O agente aceita qualquer estilo consistente.
`cargo fmt` garante consistência automática entre sessões.

**Clippy é zero-warnings obrigatório.** Um warning ignorado em uma sessão
vira problema na próxima. Manter limpo é mais barato que debugar acúmulo.

**`#[allow(...)]` existentes no projeto são intencionais:**

```rust
#![allow(clippy::missing_transmute_annotations)] // transmutes em main.rs
#![allow(static_mut_refs)]                       // apenas se necessário (V1.1)
#[allow(dead_code)]                              // stubs para V2
```

Não remover esses allows sem entender o porquê. Ver `dev/context/01_architecture.md §6`.

---

## 13. Defensive Code: Padrões Obrigatórios neste Projeto

O agente implementa caminho feliz por padrão. Os seguintes padrões são
**obrigatórios** no OpenHOTAS e devem ser aplicados sem instrução explícita:

```rust
// Contadores de erro — nunca overflow silencioso
self.error_count = self.error_count.saturating_add(1);

// Saída de todo filtro — clamp obrigatório
value.clamp(-1.0, 1.0)

// Falha de sensor — fallback para centro, nunca pânico
let rx = sens_x.read().ok();
pl_x.process(rx.unwrap_or(MT6826_ANGLE_CENTER), rx.is_some())

// Flash — validar alinhamento antes de erase/write
if !offset.is_multiple_of(SECTOR_SIZE) {
    return Err(FlashError::InvalidOffset);
}

// Divisão por zero em calibração — proteção explícita
if range == 0.0 {
    return 0.0;
}
```

---

## 14. Padrões Embassy — Armadilhas Específicas do Framework

### Acesso ao SPI — sempre via closure `with_spi`

O agente que escrever um novo sensor vai tentar acessar o SPI diretamente.
Não existe essa opção — o barramento é global, compartilhado, protegido por
`Mutex`. O único padrão aceito:

```rust
// ✅ Padrão obrigatório — qualquer acesso ao SPI no projeto
spi_bus::with_spi1(|spi| {
    self.cs.set_low();
    spi.blocking_transfer_in_place(&mut buf)
        .map_err(|_| SensorError::SpiError)?;
    self.cs.set_high();
    Ok(result)
})

// SPI0 para MCP23S17 (botões)
spi_bus::with_spi0(|spi| { ... })

// ❌ Não existe — SPI não é acessível diretamente fora da closure
let spi = get_spi1(); // não compila
```

### Tasks Embassy — tipo concreto obrigatório, sem genéricos

`#[embassy_executor::task]` tem restrições que o agente frequentemente ignora:

```rust
// ❌ Não compila em Embassy 0.10 — impl Trait não é aceito em tasks
#[embassy_executor::task]
pub async fn diagnostic_task(
    mut cdc: Sender<'static, impl Driver<'static>>,  // ← erro
) -> ! { ... }

// ✅ Tipo concreto obrigatório
#[embassy_executor::task]
pub async fn diagnostic_task(
    mut cdc: Sender<'static, Driver<'static, USB>>,  // ← correto
) -> ! { ... }

// ❌ Não compila — tasks não aceitam parâmetros genéricos
#[embassy_executor::task]
pub async fn my_task<T: SomeTrait>(arg: T) -> ! { ... }
```

Antes de criar qualquer nova task, confirmar que todos os tipos dos
parâmetros são concretos e que `'static` está presente onde necessário.
As 5 tasks existentes em `src/tasks/` são o modelo de referência.

---

## 15. O que o Agente NÃO Deve Fazer neste Projeto

```
❌ Criar task nova além das 4 existentes sem aprovação explícita
❌ Usar impl Trait ou genéricos em #[embassy_executor::task]
❌ Dividir input_task em subtasks
❌ Acessar SPI fora do padrão with_spi0 / with_spi1
❌ Adicionar dependências ao Cargo.toml sem perguntar
❌ Alterar a ordem do pipeline de sinal
❌ Redefinir constantes localmente (sempre usar constants.rs)
❌ Usar std:: ou alloc:: (projeto é no_std/no_heap)
❌ Usar f32::abs() — usar libm::fabsf()
❌ Usar println!/eprintln! — usar defmt::info!/warn!/error!
❌ Usar std::time — usar embassy_time
❌ "Corrigir" os transmute em main.rs — são intencionais
❌ Remover #[allow(dead_code)] de stubs marcados para V2
❌ Escrever em flash sem apagar o setor antes
❌ Alterar dev/context/ sem instrução explícita
❌ Adicionar lógica de throttle (projeto separado)
```

---

## Template de CLAUDE.md para este Projeto

Copiar para `CLAUDE.md` na **raiz do repositório**. Denso por design —
cada linha é lida em toda iteração. Sem redundância com `dev/context/`.

```markdown
# OpenHOTAS

Rust `no_std`/`no_heap` · Embassy 0.10 · RP2350 · Joystick HOTAS.

## Antes de qualquer edição

Ler `dev/context/` na ordem: 01 → 02 → 03 → 04 → 05.
Trabalhar em: `firmware/`

## Comandos

```sh
cargo build --release                              # build
cargo clippy --target thumbv8m.main-none-eabihf   # lint (zero-warnings)
cargo fmt --check                                  # verificar formatação
cargo fmt                                          # formatar
cargo run --release                                # flash via probe-rs
picotool uf2 convert target/.../release/openhotas -t elf out.uf2 -t uf2 --abs-block
```

## Gate de qualidade antes de flashar

```sh
cargo test -p openhotas-filters && \
cargo tree -p openhotas-filters --target x86_64-unknown-linux-gnu && \
cargo fmt --check && \
cargo clippy -p openhotas-filters -- -D warnings && \
cargo clippy -p openhotas-protocol -- -D warnings && \
cargo build --release && \
cargo clippy --target thumbv8m.main-none-eabihf -- -D warnings && \
cd firmware && cargo fmt --check
```

## no_std — substituições obrigatórias

| Proibido | Usar no lugar |
|---|---|
| `f32::abs()` | `libm::fabsf()` |
| `println!` / `eprintln!` | `defmt::info!` / `warn!` / `error!` |
| `std::time::Instant` | `embassy_time::Instant` |
| `Vec` / `HashMap` / `String` | arrays fixos / `heapless` |

## Proibições absolutas

- Não usar `std::` nem `alloc::`
- Não acessar SPI fora de `with_spi0` / `with_spi1`
- Não usar `impl Trait` ou genéricos em `#[embassy_executor::task]`
- Não "corrigir" os `transmute` em `main.rs` — são intencionais
- Não alterar a ordem do pipeline (cal→center_offset→travel→maxjump→ema→deadzone→response)
- Não dividir `input_task` · Não criar 5ª task sem aprovação
- Não escrever na flash sem apagar o setor antes
- Não adicionar dependências sem perguntar
- Não alterar `dev/context/` sem instrução explícita
- Não adicionar lógica de throttle

## Padrões obrigatórios (aplicar sempre)

- Erros: `saturating_add` em contadores, `defmt::error!` antes de retornar `Err`
- Filtros: `.clamp(-1.0, 1.0)` em toda saída
- Sensor falhou: `unwrap_or(MT6826_ANGLE_CENTER)`, `healthy = false`
- Flash: validar alinhamento de setor antes de erase/write
- Calibração: `if range == 0.0 { return 0.0; }`

## Stubs V2 — não ativar, não remover

`usb/descriptor.rs`: REPORT_ID_CONFIG = 0x02
```

---

*OpenHOTAS · Diretrizes de Código V1.3.0 · Jun/2026 (atualizado: openhotas-filters crate)
*Fontes: Akita, "Clean Code pra Agentes de IA" + "Boas práticas OSS com LLM"*
