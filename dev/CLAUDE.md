# CLAUDE.md

# OpenHOTAS Firmware Development Rules

**Projeto:** OpenHOTAS
**Versão:** V1.21
**MCU:** RP2350
**Framework:** Embassy 0.10
**Ambiente:** Rust `no_std` / `no_heap`

---

# Estrutura Oficial do Repositório

```text
OpenHotas/
├── firmware/   # Código Rust embarcado
├── hardware/   # PCB, esquemas, pinout e mecânica
└── dev/        # LLM, contexto, planos, logs e decisões
```

---

# 1. Contexto Obrigatório

Antes de qualquer alteração no firmware, ler:

```text
dev/context/

01_*
02_*
03_*
04_*
05_coding_guidelines.md
```

Ordem obrigatória:

```text
01 → 02 → 03 → 04 → 05
```

Caso exista conflito entre regras, interromper a implementação e reportar o conflito.

---

# 2. Diretório de Trabalho

O código compilável fica em:

```text
firmware/
```

Antes de modificar qualquer arquivo:

1. Localizar `firmware/Cargo.toml`.
2. Confirmar o crate alvo.
3. Mapear dependências impactadas.
4. Não assumir caminhos antigos como `firmware/firmware/` ou `llm-rules/context/`.

---

# 3. Fluxo Obrigatório

## Antes de editar

1. Ler os arquivos envolvidos.
2. Ler o contexto em `dev/context/`.
3. Entender o fluxo completo.
4. Identificar impactos.
5. Explicar o problema.
6. Apresentar plano resumido.
7. Somente então editar.

## Após editar

Executar dentro de `firmware/`:

```bash
cd firmware
cargo build --release
cargo clippy --target thumbv8m.main-none-eabihf
cargo fmt --check
```

Reportar:

* Arquivos alterados
* Impacto funcional
* Possíveis riscos
* Testes recomendados em hardware

---

# 4. Gate de Qualidade

Nenhuma tarefa é considerada concluída sem:

```bash
cd firmware &&
cargo build --release &&
cargo clippy --target thumbv8m.main-none-eabihf &&
cargo fmt --check
```

Requisitos:

* Build sem erros
* Clippy sem warnings
* Formatação válida

---

# 5. Comandos Oficiais

## Build

```bash
cd firmware
cargo build --release
```

## Lint

```bash
cd firmware
cargo clippy --target thumbv8m.main-none-eabihf
```

## Formatação

```bash
cd firmware
cargo fmt
cargo fmt --check
```

## Flash

```bash
cd firmware
cargo run --release
```

## Gerar UF2

```bash
cd firmware
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/openhotas out.uf2
```

---

# 6. Estratégia de Validação

Validação funcional ocorre somente em hardware.

Ferramentas aprovadas:

* defmt + probe-rs
* CDC Serial (V1.21+)

Não substituir testes em hardware por testes host-side.

Ao alterar lógica crítica:

* indicar pontos de teste
* sugerir logs defmt relevantes

---

# 7. Ambiente no_std

| Proibido             | Usar                    |
| -------------------- | ----------------------- |
| `f32::abs()`         | `libm::fabsf()`         |
| `println!()`         | `defmt::info!()`        |
| `eprintln!()`        | `defmt::error!()`       |
| `std::time::Instant` | `embassy_time::Instant` |
| `Vec`                | `heapless` / arrays     |
| `HashMap`            | estruturas estáticas    |
| `String`             | buffers fixos           |
| `Box`                | estruturas estáticas    |
| `Rc` / `Arc`         | ownership explícito     |

---

# 8. Proibições Absolutas

## Bibliotecas

Nunca usar:

```rust
std::
alloc::
```

## SPI

Acesso permitido apenas por:

```rust
with_spi0(...)
with_spi1(...)
```

## Embassy Tasks

Proibido em:

```rust
#[embassy_executor::task]
```

* `impl Trait`
* parâmetros genéricos

## Main

Não alterar os `transmute` existentes em `firmware/src/main.rs`.

São intencionais.

## Pipeline

Preservar rigorosamente:

```text
cal
→ maxjump
→ ema
→ deadzone
→ expo
→ response
```

## Tasks

Sem aprovação explícita:

* Não dividir `input_task`
* Não criar quinta task

## Flash

Nunca:

```text
write → erase
```

Sempre:

```text
erase → write
```

## Dependências

Não adicionar crates sem aprovação.

## Contexto LLM

Não modificar:

```text
dev/context/
```

sem instrução explícita.

## Funcionalidades

Não implementar throttle.

---

# 9. Padrões Obrigatórios

## Contadores

Preferir:

```rust
saturating_add()
saturating_sub()
saturating_mul()
```

## Tratamento de Erros

Antes de retornar erro:

```rust
defmt::error!(...)
return Err(...)
```

quando houver contexto útil.

## Filtros

Toda saída deve terminar em:

```rust
.clamp(-1.0, 1.0)
```

## Sensor Inválido

Padrão obrigatório:

```rust
unwrap_or(MT6826_ANGLE_CENTER)
healthy = false
```

## Flash

Validar alinhamento de setor antes de:

```rust
erase()
write()
```

## Calibração

Obrigatório:

```rust
if range == 0.0 {
    return 0.0;
}
```

---

# 10. Arquitetura

Priorizar:

1. Determinismo
2. Simplicidade
3. Zero alocação dinâmica
4. Baixo acoplamento
5. Responsabilidade única
6. Clareza para manutenção

Evitar:

* módulos genéricos `util.rs`
* helpers globais desnecessários
* abstrações prematuras

---

# 11. Organização de Arquivos

Meta:

```text
150–200 linhas
```

Limite recomendado:

```text
300 linhas
```

Ao ultrapassar:

* dividir responsabilidades
* criar módulos específicos

Exceção:

```text
firmware/src/main.rs
```

---

# 12. Convenções de Nomes

Evitar:

```text
data
value
val
tmp
temp
result
state
process
handler
manager
```

Preferir nomes específicos de domínio.

Objetivo:

```text
< 5 ocorrências por símbolo no repositório
```

quando viável.

---

# 13. Comentários

Preservar:

* decisões de hardware
* observações de datasheet
* justificativas arquiteturais
* histórico técnico relevante

Evitar comentários óbvios.

Explicar o **PORQUÊ** mais do que o **O QUE**.

---

# 14. Stubs V2

## Não remover

## Não ativar

### firmware/src/calibration/data.rs

```rust
start()
feed()
finish()
```

### firmware/src/calibration/cal_store.rs

```rust
save()
```

### firmware/src/config/settings.rs

```rust
save()
active_profile
```

### firmware/src/filters/*.rs

```rust
set_alpha()
set_threshold()
set_factor()
```

### firmware/src/axis/pipeline.rs

```rust
update_config()
```

### firmware/src/usb/descriptor.rs

```rust
REPORT_ID_CONFIG = 0x02
```

---

# 15. Relatório Final Obrigatório

Ao concluir uma tarefa, informar:

## Arquivos Alterados

Lista completa.

## Impacto

Mudanças funcionais realizadas.

## Riscos

Possíveis efeitos colaterais.

## Validação Recomendada

Passos para teste em hardware.

## Status

```text
Build  : PASS | FAIL
Clippy : PASS | FAIL
Fmt    : PASS | FAIL
```

---

# 16. Princípio Mestre

Quando houver dúvida:

> Escolha a solução mais simples, determinística, previsível e compatível com Embassy 0.10, RP2350 e ambiente `no_std`.
