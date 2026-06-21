# OpenHOTAS — Plano: Sistema de Versionamento do Firmware

**Versão alvo:** V1.21
**Status:** [ ] Não iniciado
**Depende de:** V1.2 compilado e limpo

---

## Contexto

O firmware não tem identificação de versão acessível em runtime. Em campo e
durante testes de hardware, não há como saber qual build está rodando sem
conectar um probe ou verificar o Cargo.toml manualmente.

| O que | Valor atual | Problema |
|---|---|---|
| `Cargo.toml` version | `0.1.0` | Não reflete a versão real (V1.2) |
| USB `device_release` | default `0x0010` | OS não vê versão do firmware |
| `CONFIG_VERSION` | `u8 = 1` | É versão do *layout de flash*, não do firmware |
| Constante de versão | Não existe | Nada em runtime identifica o build |

---

## O que NÃO muda

- `spi_bus.rs`, `sensors/`, `filters/`, `axis/` — inalterados
- `hid_gamepad.rs`, `descriptor.rs` — inalterados
- `input_task`, `hid_task`, `usb_task` — inalterados
- `CONFIG_VERSION` — é versão de layout de flash, não de firmware
- Pipeline de sinal — inalterado
- Nenhuma dependência nova de peso

---

## Proposta: 3 Camadas, Zero Custo em Runtime

### Camada 1 — `src/constants.rs`

Adicionar ao final do arquivo, antes do `pub mod tuning`:

```rust
// ── Versionamento do Firmware ─────────────────────────────────────────

/// Versão SemVer do firmware — lida do Cargo.toml em tempo de compilação
pub const FIRMWARE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Git hash curto do commit que gerou este binário
pub const FIRMWARE_GIT_HASH: &str = env!("GIT_HASH");
```

### Camada 2 — `build.rs`

Adicionar ao `build.rs` existente (após as linhas de `memory.x`):

```rust
// ── Injetar git hash ───────────────────────────────────────────────────
println!("cargo:rerun-if-changed=.git/HEAD");
if let Ok(head) = std::fs::read_to_string(".git/HEAD") {
    if let Some(ref_path) = head.strip_prefix("ref: ").map(|s| s.trim()) {
        println!("cargo:rerun-if-changed=.git/{}", ref_path);
    }
}

let git_hash = std::process::Command::new("git")
    .args(["rev-parse", "--short", "HEAD"])
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .map(|s| s.trim().to_string())
    .unwrap_or_default();
println!("cargo:rustc-env=GIT_HASH={}", git_hash);
```

Se `git` não estiver disponível (ex: CI sem checkout), `GIT_HASH` será string
vazia. O build não quebra.

### Camada 3 — USB + Diagnóstico

**`src/main.rs`** — o OS vê a versão via USB:

```rust
// Adicionar device_release ao UsbConfig existente
usb_cfg.device_release = 0x0121; // BCD: major=1, minor=21
```

**`src/tasks/diagnostic.rs`** — visível no terminal quando CDC for adicionado (V1.21):

```rust
// Linha a adicionar quando CDC estiver disponível:
// defmt::info!("OpenHOTAS {} (git:{})", FIRMWARE_VERSION, FIRMWARE_GIT_HASH);
```

---

## Arquivos Alterados

| # | Arquivo | Ação | Detalhe |
|---|---|---|---|
| 1 | `Cargo.toml` | Editar | `version = "1.2.1"` |
| 2 | `src/constants.rs` | Adicionar | `FIRMWARE_VERSION` e `FIRMWARE_GIT_HASH` |
| 3 | `build.rs` | Adicionar | Injeção do git hash (7 linhas) |
| 4 | `src/main.rs` | Adicionar | `usb_cfg.device_release = 0x0121` |

**Total:** 4 arquivos, ~13 linhas. Zero dependências novas.

---

## BCD Device Release — Referência

| Versão | `device_release` |
|---|---|
| V1.0 | `0x0100` |
| V1.1 | `0x0110` |
| V1.2 | `0x0120` |
| V1.21 | `0x0121` |
| V2.0 | `0x0200` |

**Por que não usar `CARGO_PKG_VERSION` diretamente no `device_release`?**

`device_release` exige BCD (Binary Coded Decimal). Converter `"1.2.1"` para
`0x0121` não é trivial em `no_std`. Solução: duas fontes sincronizadas manualmente:

- `Cargo.toml` → SemVer → `FIRMWARE_VERSION` (logs)
- `main.rs` → BCD manual → `device_release` (USB descriptor)

Como verificar no host:
- Windows: Gerenciador de Dispositivos → Propriedades → `bcdDevice`
- Linux: `lsusb -d 16c0:27db -v | grep bcdDevice`

---

## Checklist de Implementação

- [ ] Atualizar `version` em `Cargo.toml` para `"1.2.1"`
- [ ] Adicionar `FIRMWARE_VERSION` e `FIRMWARE_GIT_HASH` em `constants.rs`
- [ ] Adicionar injeção de git hash em `build.rs`
- [ ] Adicionar `device_release = 0x0121` em `main.rs`
- [ ] `cargo build --release` — zero erros
- [ ] `cargo clippy` — zero warnings
- [ ] Verificar `bcdDevice` no host após flash

---

## Riscos

| Risco | Prob. | Mitigação |
|---|---|---|
| `env!("GIT_HASH")` falha se não definido | Média | `build.rs` sempre define (vazio se git falhar) |
| `Cargo.toml` fora de sincronia com `device_release` | Baixa | Ambos são manuais — verificar no review |
| `env!("CARGO_PKG_VERSION")` sem campo `version` | Muito baixa | `Cargo.toml` sempre tem `version` |

---

*OpenHOTAS · Plano V1.21 Versioning · Jun/2026*
