# OpenHOTAS V1.4.0 — Atualização manual de firmware pela GUI

> Implementado em 2026-07-01 na branch `codex/v1.4-firmware-update`.

## Objetivo

Permitir que o configurador selecione um firmware UF2, coloque o RP2350 no
bootloader USB da ROM e copie o arquivo para o volume de programação sem exigir
que o usuário pressione BOOTSEL.

## Versões

- Firmware, protocolo compartilhado, CLI e GUI: `1.4.0`.
- Protocolo binário: `3.1` (novo comando aditivo).

## Protocolo e firmware

- Adicionado `Request::RebootToBootloader` ao final do enum para preservar os
  discriminantes Postcard existentes.
- O estado de reboot passou a usar `PendingReset`, distinguindo reboot normal e
  entrada no bootloader.
- O firmware envia `Response::Ack`, aguarda 100 ms e chama
  `embassy_rp::rom_data::reset_to_usb_boot(0, 0)`.
- O bootloader continua sendo o bootloader UF2 em ROM do RP2350; nenhuma região
  adicional de flash ou bootloader próprio foi criada.

## CLI

- Adicionado o comando `openhotas-cli bootloader`.
- O comando permite validar separadamente a transição para o volume RPI-RP2.

## GUI e backend Tauri

- Adicionado seletor nativo limitado a arquivos `.uf2`.
- Adicionada confirmação explícita antes do reboot e da gravação.
- O backend valida extensão, tamanho, alinhamento de 512 bytes e assinaturas de
  todos os blocos UF2 antes de reiniciar o dispositivo.
- Após o `Ack`, a porta CDC é fechada antes da enumeração do bootloader.
- O backend procura um novo volume contendo `INFO_UF2.TXT`, com timeout de 15 s
  e polling de 200 ms.
- Implementada descoberta inicial para Windows, Linux e macOS.
- O arquivo é copiado como `openhotas.uf2` e o total copiado é comparado com o
  tamanho validado.
- A tela apresenta seleção, progresso, conclusão e erro.
- Adicionado o plugin oficial de diálogo do Tauri e permissões restritas a
  abrir arquivo e confirmar operação.

## Validação executada

```bash
cargo fmt --all -- --check
cargo check -p openhotas-protocol -p openhotas-cli -p openhotas-gui
cd firmware && cargo build --release
cd gui && npm run build
git diff --check
```

Resultados:

- Protocolo, CLI e backend Tauri: OK.
- Firmware RP2350 em release: OK.
- TypeScript e bundle Vite: OK.
- Formatação e whitespace: OK.

## Validação física pendente

O fluxo completo depende de hardware e deve ser confirmado no Windows com um
Pico 2 conectado: comando CDC, aparição do volume, cópia, reboot automático e
reconexão da aplicação. O build não substitui esse teste físico.

## Fora de escopo

- Download automático de releases.
- Assinatura criptográfica do firmware.
- Rollback ou dual-bank.
- Confirmação automática da versão após o dispositivo reiniciar.

## Artefatos do GitHub Actions

- O CI de firmware converte o ELF para `openhotas.uf2` com o `picotool` oficial
  e publica o arquivo como artefato de cada execução.
- Um job dedicado em `windows-latest` gera os bundles MSI e NSIS da GUI e os
  publica como artefato de cada execução.
- O workflow acionado por tags `v*` continua anexando UF2, ELF, CLI e
  instaladores Windows à GitHub Release oficial.

## Organização dos artefatos e dependências

- Adicionadas regras globais `*.elf` e `*.uf2` ao `.gitignore` da raiz.
- O antigo `firmware/openhotas.uf2` foi retirado do índice Git, permanecendo
  apenas como arquivo local ignorado; novos binários serão distribuídos pelo
  GitHub Actions e pelas Releases.
- `Cargo.lock` e `gui/package-lock.json` foram atualizados para registrar o
  plugin oficial de diálogo do Tauri e suas dependências.
- Os schemas e manifestos gerados do Tauri foram atualizados com as permissões
  de diálogo utilizadas pelo seletor e pela confirmação de firmware.
