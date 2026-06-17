# OpenHOTAS — Configuração do Ambiente de Desenvolvimento

> Guia para você. Cobre WSL2, Rust embedded, probe-rs e flash no Pico 2.
> O `CLAUDE.md` na raiz do repositório é o guia para o agente — não este arquivo.

---

## Índice

1. [WSL2 — Setup Base](#1-wsl2--setup-base)
2. [Rust Toolchain Embedded](#2-rust-toolchain-embedded)
3. [Ferramentas de Flash e Debug](#3-ferramentas-de-flash-e-debug)
4. [USB no WSL2 — usbipd](#4-usb-no-wsl2--usbipd)
5. [Verificação do Build](#5-verificação-do-build)
6. [Estrutura do Repositório](#6-estrutura-do-repositório)
7. [Fluxo de Desenvolvimento](#7-fluxo-de-desenvolvimento)
8. [Troubleshooting](#8-troubleshooting)

---

## 1. WSL2 — Setup Base

### Requisitos no Windows

- Windows 10 (21H2+) ou Windows 11
- WSL2 com Ubuntu 22.04 ou 24.04

```powershell
# PowerShell como Admin
wsl --install -d Ubuntu-24.04

# Verificar — precisa ser versão 2
wsl --list --verbose
```

### Dependências base no Ubuntu

```bash
sudo apt update && sudo apt upgrade -y

sudo apt install -y \
    build-essential \
    pkg-config \
    libusb-1.0-0-dev \
    libssl-dev \
    libudev-dev \
    curl \
    git \
    unzip
```

---

## 2. Rust Toolchain Embedded

```bash
# Instalar rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Target do RP2350 (Cortex-M33)
rustup target add thumbv8m.main-none-eabihf

# Componentes
rustup component add clippy rustfmt

# Utilitários de binário (tamanho, símbolos)
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

Verificar:

```bash
rustc --version    # 1.96.0 stable ou mais recente
rustup target list --installed | grep thumbv8m
```

### `.cargo/config.toml` do projeto

Já existe no repositório. Define o target e o runner automaticamente:

```toml
[target.thumbv8m.main-none-eabihf]
runner = "probe-rs run --chip RP2350"

[build]
target = "thumbv8m.main-none-eabihf"

[env]
DEFMT_LOG = "debug"
```

Com isso, `cargo build` e `cargo run` funcionam sem flags adicionais.

---

## 3. Ferramentas de Flash e Debug

### probe-rs (flash + debug via probe USB)

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh \
    | sh

source "$HOME/.cargo/env"
probe-rs --version
```

### elf2uf2-rs (flash sem probe — modo bootloader)

```bash
cargo install elf2uf2-rs
```

Uso: segurar BOOTSEL ao conectar o Pico, ele aparece como drive `RPI-RP2`:

```bash
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/openhotas openhotas.uf2
# Copiar openhotas.uf2 para o drive RPI-RP2 no Windows Explorer
```

### flip-link (detecta stack overflow em link time)

```bash
cargo install flip-link
```

Adicionar ao `.cargo/config.toml` (opcional mas recomendado):

```toml
[target.thumbv8m.main-none-eabihf]
linker = "flip-link"
runner = "probe-rs run --chip RP2350"
```

---

## 4. USB no WSL2 — usbipd

WSL2 não acessa USB nativo do Windows. O `usbipd-win` faz o bridge.

### Instalar no Windows

```powershell
# PowerShell como Admin
winget install usbipd
```

### Uso — rodar antes de cada sessão com probe

```powershell
# Listar dispositivos USB
usbipd list
# BUSID  VID:PID    DEVICE
# 1-3    2e8a:000c  Picoprobe CMSIS-DAP v2
# 1-7    2e8a:0003  RP2 Boot

# Primeira vez — precisa de Admin
usbipd bind --busid 1-3

# Anexar ao WSL (toda sessão)
usbipd attach --wsl --busid 1-3
```

Verificar no WSL:

```bash
lsusb | grep -i "raspberry\|picoprobe"
probe-rs list
```

### Regras udev (evitar sudo no probe-rs)

```bash
sudo tee /etc/udev/rules.d/99-picoprobe.rules > /dev/null << 'RULES'
SUBSYSTEM=="usb", ATTRS{idVendor}=="2e8a", ATTRS{idProduct}=="000c", MODE="0666", GROUP="plugdev"
SUBSYSTEM=="usb", ATTRS{idVendor}=="2e8a", ATTRS{idProduct}=="0003", MODE="0666", GROUP="plugdev"
SUBSYSTEM=="usb", ATTRS{idVendor}=="2e8a", ATTRS{idProduct}=="0001", MODE="0666", GROUP="plugdev"
RULES

sudo usermod -aG plugdev $USER
sudo udevadm control --reload-rules && sudo udevadm trigger
newgrp plugdev
```

### Script de conveniência (Windows)

Salvar como `scripts/attach-probe.ps1`:

```powershell
$busid = (usbipd list | Select-String "Picoprobe" | ForEach-Object {
    ($_ -split '\s+')[1]
})
if ($busid) {
    usbipd attach --wsl --busid $busid
    Write-Host "Probe anexado: $busid"
} else {
    Write-Host "Probe não encontrado. Verificar conexão USB."
}
```

---

## 5. Verificação do Build

Sequência para confirmar que o ambiente está funcional:

```bash
cd openhotas/firmware

# 1. Build
cargo build --release

# 2. Lint
cargo clippy
# Deve terminar sem warnings

# 3. Tamanho do binário
cargo size --release -- -A
# Verificar se cabe no RP2350 (2MB flash, 520KB RAM)

# 4. Flash via probe
cargo run --release

# 5. Alternativa sem probe — gerar UF2
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/openhotas openhotas.uf2
```

### Checklist de hardware após primeiro flash

```
[ ] Joystick aparece no OS como "OpenHOTAS Gamepad"
[ ] Eixo X varia entre 0 e ~32767 ao girar 360°
[ ] Eixo Y varia entre 0 e ~32767
[ ] Eixo Twist varia entre 0 e ~32767
[ ] Nenhum CrcError no output defmt em operação normal
[ ] Centro ~16384 na posição mecânica central
[ ] Botões respondem — estado muda no gamepad do OS
[ ] Ciclo dentro de 500µs (diagnostic_task a cada 5s)
```

---

## 6. Estrutura do Repositório

```
openhotas/
├── CLAUDE.md                  ← lido pelo agente automaticamente
├── firmware/                  ← projeto Rust (trabalhar aqui)
│   ├── Cargo.toml
│   ├── build.rs
│   ├── memory.x
│   ├── .cargo/config.toml
│   └── src/
└── dev/                       ← contexto, planos, logs e decisões
    ├── context/               ← contratos estáveis 01→05
    ├── docs/                  ← documentação do ambiente
    ├── plan/                  ← features pendentes
    └── logs/                  ← histórico de versões
```

---

## 7. Fluxo de Desenvolvimento

### Sessão típica

```
1. Abrir terminal WSL2
2. (Opcional) Anexar probe ao WSL
3. cd firmware
4. Abrir Claude Code / Cursor na raiz do repositório
5. Descrever a tarefa — o agente lê CLAUDE.md automaticamente
6. Agente implementa → cargo clippy → cargo fmt --check
7. cargo run --release para flashar
8. Verificar no hardware com a checklist da seção 5
9. Feature concluída → mover plan/ para log/ com cabeçalho de encerramento
```

### Gate de qualidade antes de flashar

```bash
cargo build --release && \
cargo clippy --target thumbv8m.main-none-eabihf && \
cargo fmt --check
```

### Prompts úteis para o agente

**Auditoria após sessão longa:**
```
Audita o código que mudamos nessa sessão. Procura:
- código morto
- duplicação desnecessária
- valor mágico hardcoded que deveria estar em constants.rs
- violações do pipeline (cal → maxjump → ema → deadzone → expo → response)
- clippy warnings latentes
- decisões não-óbvias sem comentário de proveniência
Verifica se dev/ precisa ser atualizado com alguma decisão nova.
```

**Review antes de commitar:**
```
Antes de commitar: verifica se cargo clippy passa zero-warnings,
se cargo fmt --check não tem diff, se não há constante redefinida
localmente, e se algum arquivo ultrapassou 300 linhas.
```

**Encerramento de feature:**
```
A feature X está concluída. Mova o arquivo plan/X.md para log/X.md
e adicione o cabeçalho de encerramento com data e status do build.
Depois verifica se algum contrato em dev/context/ precisa ser
atualizado para refletir o que foi implementado.
```

---

## 8. Troubleshooting

### `error[E0463]: can't find crate for 'std'`

`#![no_std]` deve estar em `main.rs` e o target correto em `.cargo/config.toml`.

### `probe-rs: no probe found`

```bash
# Verificar se USB foi anexado
lsusb | grep -i "raspberry\|picoprobe"

# Se não aparecer — no PowerShell do Windows
usbipd attach --wsl --busid <busid>

# Se precisar de sudo mesmo com udev configurado
newgrp plugdev
```

### Build lento na primeira vez

Normal — Embassy e dependências são muitas crates. Cache incremental do
Cargo torna compilações subsequentes rápidas.

### Pico não aparece como drive RPI-RP2

Segurar **BOOTSEL** enquanto conecta o USB. O drive aparece no Windows Explorer.

### `memory.x` não encontrado

O `build.rs` copia para o diretório de output. Confirmar que
`firmware/memory.x` existe no repositório.

---

## Referências

- Embassy RP: https://docs.embassy.dev/embassy-rp/
- probe-rs: https://probe.rs/docs/
- usbipd-win: https://github.com/dorssel/usbipd-win
- The Embedded Rust Book: https://docs.rust-embedded.org/book/
- flip-link: https://github.com/knurling-rs/flip-link

---

*OpenHOTAS · Guia de Ambiente · Jun/2026*
