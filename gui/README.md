# openhotas-gui

Desktop configurator for the openhotas joystick project.
Built with **Tauri 2** (Rust backend) + **React + TypeScript** (frontend).

## Estrutura do projeto

```
gui/
├── src/                          # Frontend React
│   ├── types/
│   │   └── protocol.ts           # Tipos TypeScript espelhando os structs Rust
│   ├── lib/
│   │   └── tauri.ts              # Wrappers tipados para invoke()
│   ├── hooks/
│   │   ├── useDevicePolling.ts   # Poll de eixos + botões a ~60 Hz
│   │   ├── useDeviceConfig.ts    # Gerencia DeviceConfig com dirty flag
│   │   └── useCalibration.ts     # State machine de calibração
│   └── components/
│       ├── dashboard/            # Crosshair HUD, barras de eixo, grid de botões
│       ├── calibration/          # Fluxo de 4 passos, scope do sinal raw
│       ├── curves/               # Editor de curvas de resposta
│       └── diagnostics/          # Contadores de erro, log de protocolo
│
└── src-tauri/
    ├── Cargo.toml
    └── src/
        ├── main.rs               # Registra todos os comandos + DeviceState
        └── commands.rs           # Comandos Tauri → encode Request → CDC → decode Response
```

## Pré-requisitos

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node (recomendado: via nvm)
nvm install 20

# Tauri CLI
cargo install tauri-cli --version "^2"

# Linux: dependências de sistema para serialport
sudo apt install libudev-dev pkg-config

# macOS: nenhuma dependência extra

# Windows: nenhuma dependência extra (usa winapi)
```

## Setup

```bash
# 1. Clone / entre na pasta do projeto
cd gui

# 2. Instale dependências Node
npm install

# 3. Ajuste o path do crate no Cargo.toml se necessário
#    [dependencies]
#    openhotas-protocol = { path = "../../openhotas-protocol" }

# 4. Dev mode (hot-reload frontend + rebuild Rust quando necessário)
npm run tauri dev

# 5. Build de produção
npm run tauri build
```

## Como a comunicação funciona

```
Frontend (React)
  │
  │  invoke("get_processed_axes")
  ▼
Tauri Command (commands.rs)
  │
  │  encode_request(Request::GetProcessedAxes)
  │  → [AA 55] [LEN u16 BE] [postcard payload] [CRC16-CCITT BE]
  ▼
Serial CDC (RP2350 / Pico 2)
  │
  │  Response::ProcessedAxes { x, y, twist, unhealthy_mask }
  │  ← mesma estrutura de frame
  ▼
Tauri Command deserializa → retorna ao frontend
  │
  ▼
React state → re-render
```

## Adicionando um novo comando

1. Adicione a variante em `request.rs` do crate de protocolo.
2. Adicione o handler no firmware (Pico 2).
3. Adicione o `#[tauri::command]` em `commands.rs`.
4. Registre em `main.rs` no `invoke_handler!`.
5. Adicione o wrapper em `src/lib/tauri.ts`.

## Permissões de porta serial (Linux)

```bash
# Adicione seu usuário ao grupo dialout
sudo usermod -a -G dialout $USER
# Logout/login para aplicar
```

## Permissões Tauri (tauri.conf.json)

Certifique-se de incluir no `allowlist` ou `permissions` (Tauri v2):

```json
{
  "plugins": {
    "shell": { "open": true }
  }
}
```

A porta serial é acessada diretamente via `serialport` crate — não precisa de
permissões especiais do Tauri além do acesso padrão ao sistema.
