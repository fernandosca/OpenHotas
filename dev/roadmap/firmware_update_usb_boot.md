# Firmware Update via USB Boot (reset_to_usb_boot)

> Plano de implementação. Substitui a abordagem dual-bank do documento anterior.

---

## Por Que Esta Abordagem

O RP2350 contém na sua **ROM interna** uma função chamada `reset_to_usb_boot()`. Ela é gravada de fábrica pela Raspberry Pi Foundation e **nunca pode ser apagada ou corrompida** — está na ROM mask, não na flash.

Quando chamada, ela:
1. Desliga o firmware em execução
2. Inicializa o USB no modo Mass Storage
3. O dispositivo aparece no PC como um pen drive chamado `RPI-RP2`
4. Qualquer arquivo `.uf2` copiado para esse drive é gravado na flash automaticamente
5. Ao terminar, o RP2350 reinicia no novo firmware

**Risco de brick: zero.** Se a transferência falhar no meio, o bootloader da ROM ainda está intacto. O dispositivo sempre volta ao modo UF2 no próximo boot se o firmware estiver corrompido.

---

## Comparativo com o Plano Anterior

| Critério | Plano Anterior (dual-bank) | Esta Abordagem |
|---|---|---|
| Linhas de firmware | ~300 | ~15 |
| Linhas na GUI | ~200 | ~40 |
| Risco de brick | Alto (cópia B→A) | Zero (ROM) |
| Flash consumida | ~256KB (área B) | 0 bytes extras |
| Complexidade | Muito alta | Baixa |
| Rollback automático | Frágil | Implícito (ROM sempre presente) |
| Velocidade de update | ~16 segundos | ~3–5 segundos (USB MSC) |
| Manutenção futura | Alta | Nenhuma |

---

## Arquitetura Completa

```
┌─────────────────────────────────────────────────────┐
│  GitHub Releases                                    │
│                                                     │
│  tag: v1.2.0  →  openhotas_v1.2.0.uf2              │
└─────────────────────────────────────────────────────┘
         ↓ HTTPS (GitHub API)
┌─────────────────────────────────────────────────────┐
│  GUI Tauri                                          │
│                                                     │
│  Compara versão do firmware com latest release      │
│  Se nova: oferece update → baixa .uf2               │
│  Envia comando: REBOOT_TO_BOOTLOADER                │
└─────────────────────────────────────────────────────┘
         ↓ USB CDC Serial (protocolo postcard/CRC16)
┌─────────────────────────────────────────────────────┐
│  Firmware (usb_task)                                │
│                                                     │
│  Recebe REBOOT_TO_BOOTLOADER                        │
│  Aguarda 100ms (flush USB)                          │
│  rom_data::reset_to_usb_boot(0, 0)                 │
└─────────────────────────────────────────────────────┘
         ↓ USB desconecta e reconecta como MSC
┌─────────────────────────────────────────────────────┐
│  ROM Bootloader (imutável, gravado de fábrica)      │
│                                                     │
│  Dispositivo aparece como "RPI-RP2" (pen drive)     │
│  GUI detecta o drive, copia o .uf2                  │
│  RP2350 grava e reinicia automaticamente            │
└─────────────────────────────────────────────────────┘
```

---

## Flash Layout (sem alterações)

Nenhuma mudança no layout atual. A abordagem usa 100% da flash para o firmware.

```
0x10000000  XIP base (RP2350)
     +0x000000  Firmware principal (todo o espaço disponível)
     +0x1C0000  Config (DeviceConfig)     4KB  ← inalterado
     +0x1C1000  Calibration              4KB  ← inalterado
     +0x1C2000  Macros (futuro)          4KB  ← inalterado
```

---

## Versionamento via Cargo.toml

O `Cargo.toml` é a **única fonte de verdade** para a versão do firmware. Nenhum outro arquivo precisa ser atualizado.

```toml
# Cargo.toml
[package]
name    = "openhotas"
version = "1.2.0"   # ← única linha a editar para bump de versão
```

O firmware expõe a versão em tempo de compilação via `env!()` — custo de runtime zero:

```rust
// constants.rs

/// Versão lida do Cargo.toml em compile time — nunca desincroniza
pub const VERSION_MAJOR: u8 = env!("CARGO_PKG_VERSION_MAJOR").parse::<u8>().unwrap();
pub const VERSION_MINOR: u8 = env!("CARGO_PKG_VERSION_MINOR").parse::<u8>().unwrap();
pub const VERSION_PATCH: u8 = env!("CARGO_PKG_VERSION_PATCH").parse::<u8>().unwrap();
```

### DeviceInfo no protocolo

O firmware inclui a versão na resposta `GetDeviceInfo`, que a GUI já solicita na conexão:

```rust
// config/protocol.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub firmware_version: [u8; 3],   // [major, minor, patch]
    pub hardware_revision: u8,
    // ... outros campos existentes
}

impl DeviceInfo {
    pub fn current() -> Self {
        Self {
            firmware_version: [VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH],
            hardware_revision: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HostCommand {
    GetDeviceInfo,            // ← responde com DeviceInfo
    GetConfig,
    SetConfig(DeviceConfig),
    GetCalibration,
    SetCalibration(CalibrationData),
    RebootToBootloader,       // ← NOVO: único comando novo
}
```

---

## Implementação: Firmware

### Dependências

Sem dependências novas. `embassy-rp` já expõe `rom_data`.

```toml
# Já presente:
embassy-rp = { version = "0.10.0", features = ["rp235xa"] }
```

### Handler no USB task

```rust
// usb/hid_config.rs

use embassy_rp::rom_data;

async fn handle_command(cmd: HostCommand) -> Option<FirmwareResponse> {
    match cmd {
        HostCommand::GetDeviceInfo => {
            Some(FirmwareResponse::DeviceInfo(DeviceInfo::current()))
        }

        HostCommand::RebootToBootloader => {
            // Delay para a resposta USB ser enviada antes do reboot
            Timer::after_millis(100).await;
            rom_data::reset_to_usb_boot(0, 0);
            // Nunca chega aqui
            None
        }

        // ... handlers existentes
    }
}
```

**Parâmetros de `reset_to_usb_boot(gpio_activity_pin_mask, disable_interface_mask)`:**
- `0, 0` — sem LED de atividade, sem interfaces desabilitadas. Correto para o OpenHOTAS.

### Arquivos modificados

| Arquivo | Ação | Linhas |
|---|---|---|
| `constants.rs` | Adicionar `VERSION_*` via `env!()` | +3 |
| `config/protocol.rs` | Adicionar `DeviceInfo`, `GetDeviceInfo`, `RebootToBootloader` | +12 |
| `usb/hid_config.rs` | Adicionar handlers | +8 |

**Total firmware: ~23 linhas.**

---

## Implementação: GUI Tauri

### GitHub API — verificação de versão

A GitHub API pública não requer autenticação para repositórios públicos. Limite de 60 req/hora por IP — mais que suficiente.

```rust
// src-tauri/src/github.rs

use serde::Deserialize;

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,           // ex: "v1.2.0"
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,               // ex: "openhotas_v1.2.0.uf2"
    browser_download_url: String,
}

/// Retorna (versão_latest, url_do_uf2) se houver versão nova
pub async fn check_for_update(current: [u8; 3]) -> Result<Option<(String, String)>, String> {
    let url = "https://api.github.com/repos/seu-user/openhotas/releases/latest";

    let client = reqwest::Client::new();
    let release: GitHubRelease = client
        .get(url)
        .header("User-Agent", "openhotas-configurator")
        .send().await
        .map_err(|e| e.to_string())?
        .json().await
        .map_err(|e| e.to_string())?;

    // Parse "v1.2.0" → [1, 2, 0]
    let latest = parse_version(&release.tag_name)?;

    if latest > current {
        // Encontra o asset .uf2
        let asset = release.assets.iter()
            .find(|a| a.name.ends_with(".uf2"))
            .ok_or("Release sem arquivo .uf2")?;

        Ok(Some((release.tag_name, asset.browser_download_url.clone())))
    } else {
        Ok(None)  // Já está na versão mais recente
    }
}

fn parse_version(tag: &str) -> Result<[u8; 3], String> {
    let v = tag.trim_start_matches('v');
    let parts: Vec<u8> = v.split('.')
        .map(|p| p.parse::<u8>().map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    Ok([parts[0], parts[1], parts[2]])
}
```

### Download do .uf2

```rust
// src-tauri/src/github.rs (continuação)

use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

pub async fn download_uf2(url: &str, dest: &PathBuf) -> Result<(), String> {
    let client = reqwest::Client::new();
    let bytes = client
        .get(url)
        .header("User-Agent", "openhotas-configurator")
        .send().await
        .map_err(|e| e.to_string())?
        .bytes().await
        .map_err(|e| e.to_string())?;

    let mut file = tokio::fs::File::create(dest).await
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes).await
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

### Detecção do drive UF2

```rust
// src-tauri/src/firmware_update.rs

use std::path::PathBuf;

/// Procura pelo drive RPI-RP2 em todas as plataformas
fn find_rpi_drive() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        for letter in b'D'..=b'Z' {
            let path = PathBuf::from(format!("{}:\\", letter as char));
            if path.join("INFO_UF2.TXT").exists() {
                return Some(path);
            }
        }
        None
    }
    #[cfg(target_os = "macos")]
    {
        let path = PathBuf::from("/Volumes/RPI-RP2");
        path.exists().then_some(path)
    }
    #[cfg(target_os = "linux")]
    {
        // Tenta /mnt/RPI-RP2 e /media/<user>/RPI-RP2
        if PathBuf::from("/mnt/RPI-RP2").exists() {
            return Some(PathBuf::from("/mnt/RPI-RP2"));
        }
        // Glob em /media/ para qualquer usuário
        if let Ok(entries) = std::fs::read_dir("/media") {
            for entry in entries.flatten() {
                let candidate = entry.path().join("RPI-RP2");
                if candidate.exists() { return Some(candidate); }
            }
        }
        None
    }
}

pub async fn perform_update(uf2_path: PathBuf) -> Result<(), String> {
    // 1. Enviar comando de reboot ao firmware
    send_reboot_command().await?;

    // 2. Aguardar drive aparecer (timeout 10s)
    let drive = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        async {
            loop {
                if let Some(d) = find_rpi_drive() { return d; }
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        }
    ).await.map_err(|_| "Timeout: drive RPI-RP2 não encontrado")?;

    // 3. Copiar .uf2 para o drive
    let dest = drive.join(uf2_path.file_name().unwrap());
    std::fs::copy(&uf2_path, &dest)
        .map_err(|e| format!("Falha ao copiar firmware: {e}"))?;

    // O RP2350 reinicia automaticamente após receber o .uf2
    Ok(())
}
```

### Fluxo completo na GUI

```
Conexão estabelecida
       ↓
GUI envia GetDeviceInfo → recebe firmware_version: [1, 1, 0]
       ↓
GUI consulta GitHub API → latest: v1.2.0
       ↓
Se versão nova: badge "Atualização disponível" na UI
       ↓
Usuário clica "Atualizar para v1.2.0"
       ↓
GUI baixa openhotas_v1.2.0.uf2 para temp dir
       ↓
GUI exibe: "Reiniciando em modo de atualização..."
       ↓
GUI envia RebootToBootloader
       ↓
Firmware chama reset_to_usb_boot(0, 0)
       ↓
GUI detecta drive RPI-RP2 (polling 200ms)
       ↓
GUI copia .uf2 para o drive
       ↓
RP2350 grava e reinicia (drive desaparece)
       ↓
GUI aguarda porta serial reaparecer
       ↓
"Firmware v1.2.0 instalado com sucesso!"
```

### Arquivos criados/modificados na GUI

| Arquivo | Ação | Linhas |
|---|---|---|
| `src-tauri/src/github.rs` | Criar — API check + download | ~80 |
| `src-tauri/src/firmware_update.rs` | Criar — detect drive + copy | ~60 |
| `src-tauri/src/main.rs` | Expor comandos Tauri | ~10 |
| `src/components/FirmwareUpdate.tsx` | UI — badge + modal + progresso | ~100 |

**Total GUI: ~250 linhas.**

---

## Workflow de Release

O processo completo de publicar uma nova versão:

```bash
# 1. Bump de versão (única edição necessária)
# Cargo.toml: version = "1.2.0"

# 2. Build do firmware
cargo build --release

# 3. Converter ELF → UF2
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/openhotas openhotas_v1.2.0.uf2

# 4. Criar release no GitHub com o .uf2 como asset
gh release create v1.2.0 openhotas_v1.2.0.uf2 --title "v1.2.0" --notes "..."

# → GUIs de todos os usuários detectam a nova versão automaticamente
```

---

## Custo Total

| Recurso | Consumo |
|---|---|
| Flash código | 0 bytes (ROM call) |
| Flash dados | 0 bytes (sem área B) |
| RAM | 0 bytes extras |
| Linhas firmware | ~23 |
| Linhas GUI | ~250 |
| Infraestrutura | Zero (GitHub gratuito) |
| Risco de brick | Zero |

---

## Considerações de Segurança

**`reset_to_usb_boot()` não tem proteção por senha.** Qualquer software que consiga enviar o comando pode acionar o reboot. Para o OpenHOTAS isso é aceitável — dispositivo pessoal, sem vetor de ataque real.

Se no futuro isso for uma preocupação, a mitigação é simples: exigir que o usuário segure um botão físico no HOTAS enquanto o comando é enviado:

```rust
HostCommand::RebootToBootloader => {
    // Só executa se botão de segurança estiver pressionado
    if BUTTON_STATES.load(Ordering::Relaxed) & SAFETY_BUTTON_MASK != 0 {
        Timer::after_millis(100).await;
        rom_data::reset_to_usb_boot(0, 0);
    }
}
```

---

## Plano de Fases

### Fase 1 — Firmware (~1h)
- Adicionar `VERSION_*` em `constants.rs`
- Adicionar `DeviceInfo` e `GetDeviceInfo` ao protocolo
- Adicionar `RebootToBootloader` e handler no usb task
- Testar manualmente: enviar comando → drive aparece → copiar .uf2

### Fase 2 — GUI básica (~3h)
- `GetDeviceInfo` na conexão → exibir versão no rodapé da GUI
- Botão "Atualizar Firmware" com seleção manual de arquivo .uf2
- Detecção do drive e cópia automática
- Feedback de estado (connecting → rebooting → copying → done)

### Fase 3 — GitHub Releases (~2h)
- Verificação automática de versão via GitHub API na inicialização
- Badge "Atualização disponível" na UI
- Download automático do .uf2 com progresso
- Workflow de release documentado

---

## Teste Manual (sem GUI)

Para validar o firmware antes da GUI estar pronta, qualquer ferramenta que consiga enviar o comando serializado pelo protocolo postcard/CRC16:

```bash
# Após o firmware entrar em modo bootloader:
cp openhotas.uf2 /Volumes/RPI-RP2/   # macOS
cp openhotas.uf2 /mnt/RPI-RP2/       # Linux
copy openhotas.uf2 E:\               # Windows

# O RP2350 grava e reinicia sozinho em ~3 segundos
```

---

*OpenHOTAS · Plan · Jun/2026*
