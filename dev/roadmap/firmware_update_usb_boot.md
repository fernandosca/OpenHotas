# Firmware Update Manual via USB Boot

> Plano firme para implementar update seguro usando o bootloader ROM do RP2350.

---

## Objetivo

Permitir que a GUI coloque o Pico 2 em modo bootloader USB e copie um arquivo
`.uf2` escolhido manualmente pelo usuario.

Esta fase nao inclui verificacao automatica no GitHub, download automatico ou
comparacao com releases remotas. Essas ideias ficam em rascunho separado.

---

## Decisao

Usar `reset_to_usb_boot(0, 0)` da ROM interna do RP2350.

Motivos:

- bootloader fica em ROM mask, nao na flash
- nao consome espaco de flash
- nao exige dual-bank
- nao exige copia manual entre regioes de flash
- reduz risco de brick
- usa fluxo UF2 padrao do Pico

---

## Fluxo Esperado

```text
Usuario seleciona arquivo .uf2 na GUI
        ↓
GUI envia comando RebootToBootloader via CDC
        ↓
Firmware responde Ack
        ↓
Firmware aguarda ~100 ms
        ↓
Firmware chama reset_to_usb_boot(0, 0)
        ↓
Dispositivo reconecta como drive RPI-RP2
        ↓
GUI detecta o drive
        ↓
GUI copia o .uf2 selecionado
        ↓
ROM grava firmware e reinicia automaticamente
```

---

## Estado Atual do Projeto

Parte da base ja existe:

- `Request::GetInfo` ja substitui o antigo conceito de `GetDeviceInfo`
- `Response::Info(DeviceInfo)` ja retorna versao de firmware e protocolo
- `FIRMWARE_VERSION` ja vem de `env!("CARGO_PKG_VERSION")`
- GUI Tauri ja envia comandos CDC por `gui/src-tauri/src/commands.rs`
- `Request::Reboot` ja existe, mas faz reboot normal da aplicacao

O comando novo deve ser separado de `Request::Reboot`.

---

## Mudancas de Protocolo

Arquivo:

```text
crates/openhotas-protocol/src/request.rs
```

Adicionar:

```rust
/// Reboot into RP2350 ROM USB bootloader (RPI-RP2 UF2 drive).
RebootToBootloader,
```

Observacoes:

- manter `Request::Reboot` como reboot normal
- `RebootToBootloader` deve retornar `Response::Ack` antes do reset
- atualizar `PROTOCOL_VERSION_MINOR` ou `PROTOCOL_VERSION_MAJOR`, conforme a
  politica adotada para novo comando no protocolo

---

## Mudancas no Firmware

Arquivos principais:

```text
firmware/src/tasks/cdc.rs
firmware/src/tasks/cdc_handlers.rs
```

Implementacao recomendada:

- adicionar estado `pending_bootloader: bool` ao lado de `pending_reboot`
- tratar `Request::RebootToBootloader` no handler de escrita
- responder `Ack`
- apos enviar o frame de resposta, aguardar ~100 ms
- chamar `embassy_rp::rom_data::reset_to_usb_boot(0, 0)`

Exemplo conceitual:

```rust
Request::RebootToBootloader => {
    *pending_bootloader = true;
    Response::Ack
}
```

No loop CDC, apos enviar a resposta:

```rust
if pending_bootloader {
    Timer::after_millis(100).await;
    embassy_rp::rom_data::reset_to_usb_boot(0, 0);
}
```

Parametros:

```text
gpio_activity_pin_mask = 0
disable_interface_mask = 0
```

---

## Mudancas na CLI

Adicionar comando simples para teste antes da GUI:

```text
openhotas-cli --port <porta> bootloader
```

Objetivo:

- validar firmware sem depender da GUI
- confirmar que o drive `RPI-RP2` aparece
- permitir teste manual copiando `.uf2`

---

## Mudancas na GUI

Arquivos provaveis:

```text
gui/src-tauri/src/commands.rs
gui/src-tauri/src/main.rs
gui/src/lib/tauri.ts
gui/src/components/settings/SettingsPage.tsx
```

Comportamento esperado:

- botao em Configuracoes para atualizar firmware manualmente
- seletor de arquivo `.uf2`
- confirmacao antes de reiniciar em bootloader
- envio de `Request::RebootToBootloader`
- deteccao do drive `RPI-RP2`
- copia do `.uf2` para o drive
- feedback de estado para o usuario

Estados minimos:

```text
idle
file_selected
rebooting_to_bootloader
waiting_for_rpi_drive
copying_uf2
done
error
```

---

## Deteccao do Drive RPI-RP2

Prioridade inicial: Windows.

No Windows, procurar unidades `D:\` ate `Z:\` que contenham:

```text
INFO_UF2.TXT
```

Suporte posterior pode adicionar:

```text
macOS: /Volumes/RPI-RP2
Linux: /media/<usuario>/RPI-RP2 ou /mnt/RPI-RP2
```

Timeout inicial recomendado:

```text
10 segundos
```

Intervalo de polling:

```text
200 ms
```

---

## Fora do Escopo Desta Fase

- consultar GitHub Releases
- baixar `.uf2` automaticamente
- comparar versao local com ultima release
- mostrar badge de update disponivel
- assinatura/verificacao criptografica do arquivo
- rollback automatico
- dual-bank

---

## Riscos e Mitigacoes

| Risco | Mitigacao |
|---|---|
| Usuario seleciona arquivo errado | Confirmar nome e extensao `.uf2` antes de copiar |
| Drive RPI-RP2 demora a aparecer | Timeout claro e botao para tentar novamente |
| Porta serial desaparece no reboot | Fechar conexao antes de aguardar o drive |
| Copia falha | Mostrar erro e orientar copiar manualmente |
| Comando acidental | Confirmacao antes de reiniciar em bootloader |

---

## Plano de Implementacao

### Fase 1 - Firmware e CLI

- adicionar `Request::RebootToBootloader`
- implementar handler no firmware
- adicionar comando CLI `bootloader`
- teste manual: comando CLI faz aparecer `RPI-RP2`

### Fase 2 - GUI Manual

- selecionar `.uf2`
- enviar comando de bootloader
- detectar drive
- copiar arquivo
- mostrar estado final

### Fase 3 - Validacao

- testar no Windows
- testar falha por timeout
- testar arquivo invalido
- testar reconexao apos update

---

## Criterio de Pronto

- CLI consegue colocar o dispositivo em modo `RPI-RP2`
- GUI consegue copiar um `.uf2` selecionado manualmente
- falhas comuns exibem erro claro
- `Request::Reboot` normal continua funcionando
- CI passa para protocolo, firmware, CLI e GUI

---

*OpenHOTAS - Roadmap - Firmware Update Manual via USB Boot*
