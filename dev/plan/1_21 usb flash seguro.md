# OpenHOTAS — Plano Revisado: Flash Driver Seguro (Safe Rust)

**Versão alvo:** V1.21
**Status:** [ ] Não iniciado
**Depende de:** V1.2 compilado e limpo

---

# Contexto

A versão V1.2 eliminou os principais pontos de risco relacionados a acesso concorrente e ponteiros brutos nos caminhos críticos de aquisição e processamento de dados.

Entretanto, o módulo `src/storage/flash.rs` ainda utiliza o padrão legado baseado em:

```rust
static mut FLASH_INSTANCE: Option<Flash<...>>;
```

com supressão explícita de lints através de:

```rust
#[allow(static_mut_refs)]
```

Este plano migra o subsistema de persistência da flash interna do RP2350 para uma implementação alinhada aos princípios adotados na V1.2:

* Safe Rust por padrão.
* Eliminação de `static mut`.
* Exclusão mútua garantida pelo compilador.
* Validação explícita de limites.
* Isolamento mínimo de código `unsafe`.

---

# Objetivos

* Eliminar completamente o uso de `static mut`.
* Remover `#[allow(static_mut_refs)]`.
* Preservar a API pública existente.
* Manter compatibilidade com Embassy RP 0.10.
* Melhorar diagnósticos de erro.
* Garantir validação de limites antes de operações destrutivas.

---

# O que NÃO muda

* Assinaturas públicas:

  * `init()`
  * `read_flash()`
  * `write_flash()`
  * `erase_sector()`
  * `crc32()`

* Operações bloqueantes do Embassy.

* Tamanho de setor de 4096 bytes.

* Leitura via XIP.

* Proibição de escrita/apagamento dentro de ISR.

---

# Arquivos Alterados

| Arquivo                | Ação               |
| ---------------------- | ------------------ |
| `src/storage/flash.rs` | Reescrita completa |

---

# Implementação

## Tipo do Driver

```rust
pub type OpenHotasFlash =
    Flash<'static, FLASH, Blocking, { 2 * 1024 * 1024 }>;
```

---

## Estado Global Seguro

Substituir:

```rust
static mut FLASH_INSTANCE: Option<OpenHotasFlash>;
```

por:

```rust
use core::cell::RefCell;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub static FLASH_INSTANCE:
    Mutex<CriticalSectionRawMutex,
    RefCell<Option<OpenHotasFlash>>> =
    Mutex::new(RefCell::new(None));
```

### Benefícios

* Sem mutabilidade global insegura.
* Sem necessidade de suppress de lint.
* Exclusão mútua validada pelo compilador.
* Consistente com a arquitetura utilizada no barramento SPI.

---

## Inicialização

```rust
pub fn init(flash: OpenHotasFlash) {
    FLASH_INSTANCE.lock(|state| {
        *state.borrow_mut() = Some(flash);
    });
}
```

---

## Helper de Acesso

```rust
fn with_flash<R>(
    f: impl FnOnce(&mut OpenHotasFlash)
        -> Result<R, FlashError>,
) -> Result<R, FlashError> {
    FLASH_INSTANCE.lock(|state| {
        let mut flash_ref = state.borrow_mut();

        let flash = flash_ref
            .as_mut()
            .ok_or(FlashError::NotInitialized)?;

        f(flash)
    })
}
```

---

# Revisão dos Erros

Substituir:

```rust
pub enum FlashError {
    ReadError,
    WriteError,
    EraseError,
    InvalidOffset,
}
```

por:

```rust
pub enum FlashError {
    ReadError,
    WriteError,
    EraseError,
    InvalidOffset,
    OutOfBounds,
    NotInitialized,
}
```

### Justificativa

Permite diferenciar:

* Driver não inicializado.
* Offset inválido.
* Acesso fora da região válida da flash.

Facilita diagnóstico durante testes e manutenção.

---

# Validação de Limites

Adicionar:

```rust
const FLASH_SIZE: u32 = 2 * 1024 * 1024;
```

Helper:

```rust
fn validate_range(
    offset: u32,
    len: usize,
) -> Result<(), FlashError> {
    let end = offset
        .checked_add(len as u32)
        .ok_or(FlashError::OutOfBounds)?;

    if end > FLASH_SIZE {
        return Err(FlashError::OutOfBounds);
    }

    Ok(())
}
```

---

# Leitura

```rust
pub fn read_flash(
    offset: u32,
    buf: &mut [u8],
) -> Result<(), FlashError> {

    validate_range(offset, buf.len())?;

    let ptr =
        (0x10000000u32 + offset) as *const u8;

    for (i, byte) in buf.iter_mut().enumerate() {
        *byte = unsafe {
            core::ptr::read_volatile(ptr.add(i))
        };
    }

    Ok(())
}
```

---

# Apagamento

## Compatibilidade de Toolchain

Substituir:

```rust
offset.is_multiple_of(SECTOR_SIZE)
```

por:

```rust
offset % SECTOR_SIZE == 0
```

### Motivo

Embora versões modernas do Rust ofereçam `is_multiple_of()`, a operação modular explícita possui compatibilidade universal com:

* Ambientes `no_std`.
* Toolchains embarcados mais antigos.
* Diferentes pipelines de CI.

---

## Implementação

```rust
pub fn erase_sector(
    offset: u32,
) -> Result<(), FlashError> {

    if offset % SECTOR_SIZE != 0 {
        return Err(FlashError::InvalidOffset);
    }

    validate_range(
        offset,
        SECTOR_SIZE as usize,
    )?;

    with_flash(|flash| {
        flash
            .blocking_erase(
                offset,
                offset + SECTOR_SIZE,
            )
            .map_err(|_| FlashError::EraseError)
    })
}
```

---

# Escrita

Adicionar validação prévia:

```rust
pub fn write_flash(
    offset: u32,
    data: &[u8],
) -> Result<(), FlashError> {

    validate_range(
        offset,
        data.len(),
    )?;

    with_flash(|flash| {
        flash
            .blocking_write(
                offset,
                data,
            )
            .map_err(|_| FlashError::WriteError)
    })
}
```

---

# Código Unsafe

Apenas um bloco `unsafe` permanece:

```rust
unsafe {
    core::ptr::read_volatile(ptr.add(i))
}
```

### Justificativa

A região XIP da flash é memória mapeada por hardware e requer leitura volátil.

Nenhum outro bloco `unsafe` deverá permanecer no módulo.

---

# Checklist de Revisão

## Segurança

* [ ] Nenhum `static mut`.
* [ ] Nenhum `#[allow(static_mut_refs)]`.
* [ ] Apenas um bloco `unsafe`.
* [ ] Unsafe restrito à leitura XIP.
* [ ] Sem ponteiros mutáveis globais.

## Funcionalidade

* [ ] Driver inicializa corretamente.
* [ ] Leitura validada.
* [ ] Escrita validada.
* [ ] Apagamento validado.
* [ ] Erros retornados corretamente.

## Embassy RP 0.10

* [ ] Confirmar alinhamento exigido por `blocking_write()`.
* [ ] Confirmar tamanho mínimo de escrita.
* [ ] Confirmar semântica de intervalo de `blocking_erase()`.
* [ ] Confirmar comportamento durante execução em XIP.

## Testes

* [ ] Leitura após boot.
* [ ] Escrita de configuração.
* [ ] Leitura da configuração gravada.
* [ ] CRC válido após reboot.
* [ ] Falha ao gravar fora da região válida.
* [ ] Falha ao apagar setor desalinhado.
* [ ] Falha quando driver não inicializado.

---

# Critérios de Conclusão

A V1.21 será considerada concluída quando:

* Nenhum `static mut` existir em `flash.rs`.
* Nenhum suppress de lint for necessário.
* Todos os acessos mutáveis à flash forem realizados via `Mutex`.
* Todas as operações validarem limites antes do acesso.
* O módulo compilar sem warnings relacionados a segurança.
* Os testes de persistência passarem após reboot do dispositivo.
