# OpenHOTAS — Changelog

Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).
Histórico consolidado a partir dos build logs em `dev/logs/`.

<!--
INSTRUÇÃO PARA IA — leia antes de adicionar qualquer entrada neste arquivo.

Este arquivo é o CHANGELOG do projeto OpenHOTAS. Contém APENAS mudanças de código/comportamento
visíveis — o que foi adicionado, alterado, corrigido ou removido. NÃO contém decisões de design
(isso vai em DECISIONS.md) nem riscos técnicos aceitos (isso vai em RISKS.md).

Ao processar o resumo de uma sessão de trabalho:
1. Toda entrada nova entra em "## [Unreleased]", na subseção correta: Added / Changed / Fixed / Removed.
   Se a subseção não existir ainda em [Unreleased], crie-a.
2. Formato de cada linha: "- [área] descrição objetiva, técnica, uma frase."
   Área = firmware / hardware / pcb / configurador (gui) / cli / crates / protocol / geral.
3. NUNCA invente número de versão. Versão só é atribuída manualmente pelo autor no momento do
   release (quando ele renomeia [Unreleased] para [X.Y.Z] - data e abre um [Unreleased] novo vazio).
4. NUNCA misture decisão de design ou risco técnico aqui — se o resumo da sessão contiver esse tipo
   de conteúdo, aponte que ele pertence a DECISIONS.md ou RISKS.md em vez de inserir aqui.
5. Item vago demais pra classificar com confiança ("ajustes gerais", "correções diversas") não deve
   virar entrada — peça mais detalhe ou ignore.
6. Preserve nomes de arquivo, função, struct e valores exatamente como aparecem no resumo da sessão.
7. Não reescreva nem reordene entradas já existentes de versões fechadas — só adicione em [Unreleased].
-->

---

## [Unreleased]

### Added

### Changed

### Fixed

### Removed

---

## [1.4.2] - 2026-07-04

### Added

- [gui] `WindowBar` exibe tela ativa e estado de conexão com indicador semântico integrado à área de arraste
- [gui] `AxisTabTrigger` centraliza tokens e variantes visuais das abas de eixo em Dashboard, Curves e Calibration
- [gui] `UnsavedChangesBar` centraliza erros, estado pendente e ações de salvar/descartar configuração
- [gui] Release Windows publica executável portable x64 e arquivo SHA-256 junto aos instaladores MSI/NSIS
- [firmware] `SensorHealth` enum em `sensors/mod.rs` (Healthy/Degraded/Failed) — padroniza distinção entre falha de barramento e falha de sensor individual entre MT6826S e MCP23S17
- [firmware] `Sensor::health()` no trait `Sensor` — expõe estado de saúde para diagnóstico via CDC
- [firmware] `runtime_health_check()` em `mcp23s.rs` — readback periódico de IOCON (~1s) para detectar MCP23S17 morto após o boot
- [firmware] `log_health_transition()` em `input.rs` — loga transições Healthy→Degraded→Failed via defmt sem spam
- [firmware] `TODO(hardware-fix)` em `hid_gamepad.rs` — documenta swap temporário de X/Y que deve ser removido quando a PCB definitiva chegar

### Changed

- [geral] Versões de firmware e GUI atualizadas de `1.4.1` para `1.4.2`
- [gui] WebView aplica CSP local-only para bloquear scripts, frames, formulários e conexões externas sem impedir IPC do Tauri
- [firmware] `from_utf8_unchecked` substituído por `from_utf8().unwrap()` em `chip_id_serial_static()` — invariante garante UTF-8 válido (ASCII hex), elimina bloco unsafe desnecessário
- [geral] CI valida o backend Tauri com `cargo check` no Windows e reserva a geração de MSI/NSIS para workflows de release
- [gui] Diagnostics separa métricas de runtime, erros e sensores em três seções responsivas com status semântico
- [gui] Cards e shells das páginas usam largura máxima, espaçamento inicial e alinhamento consistentes entre telas
- [gui] Cards, sidebar, barra superior e controle de conexão adotam superfícies sem bordas externas e cantos mais discretos

### Fixed

- [gui] `WindowBar` usa exclusivamente regiões de arraste nativas do Tauri, evitando comportamento irregular causado pela chamada simultânea de `startDragging()`
- [gui] Grade de 32 botões passa a usar 16 colunas em telas largas por meio de `gridTemplateColumns.16`
- [gui] Status de sensores em Diagnostics avalia `healthy` e `error_count` sem converter textos com `FAULT` para número
- [crates] Guarda contra `f32::NAN` em `MaxJump`, `Ema`, `Deadzone` e `ResponseCurve` — NaN bypassava `clamp()` e podia corromper estado interno dos filtros

### Removed

---

## [1.4]

### Added
- [firmware] `Request::RebootToBootloader` — comando aditivo ao final do enum para preservar discriminantes Postcard existentes
- [firmware] Estado `PendingReset` — distingue reboot normal e entrada no bootloader
- [firmware] `reset_to_usb_boot(0, 0)` via `embassy_rp::rom_data` — coloca RP2350 no bootloader USB da ROM
- [cli] `openhotas-cli bootloader` — valida transição para o volume RPI-RP2
- [gui] Seletor nativo limitado a arquivos `.uf2`
- [gui] Confirmação explícita antes do reboot e da gravação
- [gui] Validação backend: extensão, tamanho, alinhamento de 512 bytes e assinaturas de blocos UF2
- [gui] Descoberta de volume para Windows, Linux e macOS (timeout 15 s, polling 200 ms)
- [gui] Tela com seleção, progresso, conclusão e erro
- [gui] Plugin oficial de diálogo do Tauri + permissões restritas
- [gui] Marcador em tempo real no gráfico da curva (saída processada pelo firmware)

### Changed
- [protocol] Versão binária: 3.1 (novo comando aditivo)
- [gui] Logo sidebar substituída por imagem provisória de aviação
- [gui] Barra superior personalizada — arraste da janela habilitado; moldura nativa removida
- [gui] Permissões Tauri liberadas para fechar, minimizar e iniciar arraste
- [gui] Atualização imediata de estado após conectar/desconectar
- [gui] Polling de eixos e estatísticas disparado após conexão
- [gui] Recarga de configuração ativa ao conectar e após calibração
- [gui] Erros de calibração descartam sessão pendente no firmware (finalização como cancelamento seguro)
- [gui] Salvamento de curvas não sobrescreve calibração recente com cópia local antiga
- [gui] Grade de botões exibe índices `0..31` alinhados ao protocolo/firmware
- [gui] Compensação de eixos invertidos na visualização do marcador de curva
- [firmware] Revisão completa de comentários em todos os 32 arquivos `.rs` do firmware + crates (+930 linhas)
- [geral] `Cargo.lock` e `gui/package-lock.json` atualizados com plugin de diálogo Tauri

### Removed
- [gui] `firmware/openhotas.uf2` retirado do índice Git — binários distribuídos pelo CI/Releases
- [geral] Regras globais `*.elf` e `*.uf2` adicionadas ao `.gitignore` da raiz

---

## [1.3]

### Added
- [firmware] Axis-to-Button — mapeia posição de eixo para botão virtual no report HID (threshold em permille, direção, button_index)
- [firmware] Center Offset — ajuste fino do ponto zero em permille (-200..200) sem recalibrar
- [firmware] Validação de colisão — botões virtuais não colidem entre si; botão virtual e físico compõem por OR no mesmo bit HID
- [firmware] `openhotas-filters` crate — extração de lógica pura (EMA, Deadzone, MaxJump, ResponseCurve, Calibration, CRC32) com 29 testes unitários, `#![no_std]`, dependência apenas de `libm`
- [firmware] Double-buffer com geração + CRC32 para power-fail safety em `config/stored_config_v2.rs` — boot lê ambos slots, usa o de maior geração; save escreve no slot inativo com geração = atual + 1
- [firmware] Compile-time assertion para `frame_buf` em `tasks/cdc.rs`
- [firmware] Snapshot atômico com `critical_section::with` na calibração
- [firmware] `reset_max_cycle()` — retorna pico da janela anterior e reseta contador
- [firmware] Watchdog de hardware via `embassy_rp::watchdog::Watchdog` — timeout de 2000 ms, alimentado a cada ciclo em `input_task`
- [firmware] Calibração circular de 15 bits — cada amostra e extremo representados pela menor distância assinada em relação ao centro
- [firmware] Testes automatizados para passagem pelo zero, centro/limites corretos, rejeição de extremos no mesmo lado (32/32 testes em `openhotas-filters`, 8/8 em `openhotas-protocol`)
- [firmware] Novo erro interno `SensorError::NotPresent` — diferencia ausência física de falha de transporte
- [firmware] Safety comment em `axis_to_i16` em `usb/hid_gamepad.rs`
- [firmware] Proteção contra truncation em `as_micros()` em `tasks/input.rs`
- [crates] `AxisDirection`, `AxisToButtonConfig`, `center_offset_permille` em `openhotas-protocol/src/config.rs`
- [gui] Controles visuais para `center_offset_permille` e `axis_to_button` em `Dashboard.tsx`
- [cli] `--center-offset`, `--axis-to-button` em `commands.rs`
- [firmware] `serial USB contract` — formato `"OH{:016X}"` (18 bytes) documentado em `04_software_contracts.md §8.1`
- [firmware] Dependência `static_cell = "2.1.1"` em `Cargo.toml`

### Changed
- [firmware] `openhotas-protocol`/gui/cli versionados como `1.3.0`
- [protocol] `PROTOCOL_VERSION_MAJOR`: 2 → 3, depois → 3.1 (comando aditivo)
- [firmware] SERIAL_STR refactor — raw pointer writes byte-a-byte → buffer local + `copy_nonoverlapping`
- [firmware] `main.rs` — 7 `static mut` migrados para `StaticCell`: `DD`, `CD`, `BD`, `CB`, `HS`, `CDC_STATE`, `SERIAL_STR`
- [firmware] `sensors/mt6826.rs` — constante `MT6826_CMD_READ_ANGLE` em vez de magic number `0xA0`
- [firmware] `constants.rs` — constantes `STORED_V2_SLOT_A`/`STORED_V2_SLOT_B`
- [firmware] `Cargo.toml` — `embassy-time` atualizado de 0.4 → 0.5
- [firmware] `tasks/input.rs` — `wdt.feed(...)` a cada iteração do loop
- [firmware] Driver MT6826S — setup de 1 µs entre `CSN LOW` e primeiro clock; hold de 1 µs antes de `CSN HIGH`
- [firmware] `input_task` — aguarda 5 ms antes da primeira leitura (power-up)
- [firmware] Conversor `elf2uf2-rs` removido — fluxo oficial passou a usar `picotool uf2 convert ... --abs-block`
- [firmware] Frequência SPI final: 1 MHz
- [firmware] `max_jump_raw` passa a ser escalado pelo curso circular calibrado
- [firmware] Margem experimental de 2 µs após `CSN HIGH` testada e removida
- [firmware] Revisão do linker RP2350 — restaurado `link.x` oficial do `cortex-m-rt 0.7.5`
- [firmware] Revisão completa de comentários em todos os 32 arquivos `.rs` do firmware + crates (+930 linhas)
- [firmware] `filters/mod.rs`, `calibration/mod.rs`, `storage/flash.rs` → re-exportam tipos do crate `openhotas-filters`

### Fixed
- [firmware] CHIP_ID — endereços `0x00010040`/`0x00010044` incorretos — substituído por `otp::get_chipid()`
- [firmware] Serial USB poderia ser igual em todas as placas — corrigido com OTP
- [firmware] `memory.x` com layout de RP2040 incorreto — FLASH alterada para `ORIGIN = 0x10000000`
- [firmware] MISO flutuante do MT6826S produzia frames zerados com CRC falsamente positivo — pull-up interno habilitado
- [firmware] Escritas SPI no MCP23S17 retornavam sucesso sem chip conectado — inicialização agora lê de volta os registradores
- [firmware] CS pins dos encoders X e Y invertidos fisicamente no hardware de teste — contornado via `set-axis --invert`
- [firmware] `#[allow(static_mut_refs)]` (3 ocorrências) removido — padrões `unsafe` substituídos por `StaticCell::init()`

### Removed
- [firmware] Constante morta `REPORT_ID_GAMEPAD` de `constants.rs`
- [firmware] Constantes `CHIP_ID_HI`/`CHIP_ID_LO` de `main.rs`
- [firmware] Instrução temporária `sensor-frames` removida após causa física identificada

---

## [1.28]

### Added
- [gui] Infraestrutura de temas — `ThemeProvider` com `data-theme`, persistência em `localStorage`, fallback `HUD`
- [gui] `gui/src/theme/themes.ts` — registro tipado dos temas disponíveis
- [gui] Tokens semânticos em `gui/src/theme/theme.css`
- [gui] Tema `HUD` e tema `Glass Cockpit`
- [gui] Seletor de aparência na tela `Configurações`

### Changed
- [gui] `colors.ts` — valores estáticos substituídos por resolução via `getComputedStyle`
- [gui] `tailwind.config.ts` — consome variáveis do contrato de temas
- [gui] Canvas e gráficos redesenham quando tema muda
- [gui] Storybook — stories migradas para tokens semânticos

---

## [1.27]

### Changed
- [gui] Tela `Configuração` removida da navegação — controles transferidos para `Eixos` e `Botões`
- [gui] `Dashboard.tsx` — controles de eixo consolidados em card único
- [gui] `ButtonsPage.tsx` — card de configuração movido para esta tela
- [gui] `CurvePage.tsx` — gráfico simplificado, presets: Linear, Suave, Centro, S-curve
- [gui] `Calibration.tsx` — consolidada em único card
- [gui] `Diagnostics.tsx` — reorganizada (Runtime stats no topo, ações na parte inferior)

### Removed
- [gui] Arquivos e stories antigos da tela `Configuração`
- [gui] Referências de edição visual por Puck

---

## [1.26]

### Added
- [gui] `ButtonsPage.tsx` — grid de 32 botões, lista ativa, masks, debounce
- [cli] Handshake obrigatório — `connect_to()` valida `protocol_major`, `axis_count`, `button_count`
- [cli] Auto-detect por identidade — `connect()` valida handshake `GetInfo` em cada porta

### Changed
- [gui] `App.tsx`, `Dashboard.tsx`, `CurvePage.tsx`, `Calibration.tsx`, `Diagnostics.tsx` — reorganização/consolidação de telas
- [firmware] `tasks/input.rs` — `Ticker::every(500µs)` cadencia o loop
- [firmware] `CaptureCalibrationPoint` rejeita captura se eixo unhealthy
- [firmware] Sessão de calibração limpa ao desconectar CDC

---

## [1.25]

### Added
- [firmware] Response Curve piecewise linear com 5 pontos de controle
- [crates] `CurvePoint`, `ResponseCurveData` no protocol crate
- [gui] `CurvePage.tsx` — piecewise linear com presets (Linear, Suave, Centro, S-curve)

### Changed
- [firmware] `filters/response_curve.rs` reescrito — piecewise linear em vez de curva cúbica simples
- [firmware] `expo` substituído por `response_p1`, `response_p3` em `axis/mod.rs` e pipeline
- [protocol] `expo_i16` removido de `AxisConfig`, `PROTOCOL_VERSION_MAJOR` 1 → 2
- [gui] `protocol.ts` atualizado, `CurveEditor.tsx` redesenhado

### Fixed
- [firmware] Protocol version mismatch — `protocol.ts` `1` → `2`
- [firmware] CRC8 overflow — `wrapping_shl(1)`
- [firmware] Deadzone div/zero — guard `threshold >= 1.0`
- [firmware] SPI bus `expect()` → `Result<R, SpiBusError>`
- [firmware] `read_flash` sem checagem de init antes de XIP
- [firmware] `GIT_HASH` sem fallback — `option_env!` + `"unknown"`

### Removed
- [firmware] `filters/expo.rs` — subsumido por response curve
- [firmware] `REPORT_ID_CONFIG` de `constants.rs`

---

## [1.24]

### Changed
- [firmware] Macro `track_delta!` para tracking de deltas de contadores de erro

### Fixed
- [firmware] MaxJump não reseta ao reabilitar eixo — `last_valid` congelado causava trava no centro
- [firmware] `max_jump_raw` não considerava range de calibração
- [firmware] EMA não reseta ao mudar calibração — artefatos curtos na transição

---

## [1.23]

### Added
- [firmware] Helper `signal_latest_config()` — drena canal cheio e retenta (latest-wins)
- [cli] `openhotas-cli` — 13 comandos funcionais (info, get-config, set-axis, save, calibrate, etc.)

### Changed
- [firmware] Reboot com Ack + delay de 100ms antes de `SCB::sys_reset()`

### Fixed
- [firmware] `CaptureCalibrationPoint` rejeitava `raw == 0`
- [firmware] `expo_i16` negativo ignorado
- [firmware] `save_config` usava 4KB na stack — buffer otimizado
- [firmware] `Channel::try_send` silencioso quando canal cheio — corrigido (latest-wins)
- [firmware] CRC protocolo vs CRC sensor — separados

### Removed
- [firmware] `cal_store.rs`, `settings.rs`, `sensor_health.rs` — substituídos por `stored_config_v2.rs` / `runtime_stats.rs`
- [firmware] Constantes `CALIB_OFFSET`, `CONFIG_OFFSET`, `MAGIC_*` — layout V1 obsoleto

---

## [1.22]

### Added
- [firmware] Configuration Protocol — configuração/calibração via USB CDC, protocolo binário request/response
- [firmware] `src/tasks/cdc.rs`, `cdc_handlers.rs`
- [firmware] `src/config/runtime.rs` — `RuntimeConfig`, `CONFIG_SIGNAL`
- [firmware] `src/config/stored_config_v2.rs` — layout V2 (MAGIC + VERSION + PAYLOAD + CRC32)
- [crates] `crates/openhotas-protocol/` — lib, config, request, response, diagnostics, error, frame, version
- [crates] Frame format: SOF + LEN + PAYLOAD (postcard) + CRC16-CCITT

### Changed
- [firmware] `usb/descriptor.rs` — Report ID removido, Logical Minimum ajustado para i16 assinado
- [firmware] `diagnostic.rs` desacoplada do CDC, voltou a logar via `defmt`

---

## [1.21]

### Added
- [firmware] Sistema de versionamento em três camadas: Cargo.toml (SemVer), build.rs (git hash), USB descriptor (BCD)
- [firmware] `FIRMWARE_VERSION`, `FIRMWARE_GIT_HASH` em `constants.rs`
- [firmware] `build.rs` — injeção de `GIT_HASH` via `env!` com `rerun-if-changed`
- [firmware] CDC ACM — canal de debug com banner, telemetria a cada 5s, alerta `WARN:max_cycle`
- [firmware] `runtime_stats.rs` — atomics tornados `pub`

### Changed
- [firmware] `storage/flash.rs` reescrita — `static mut` → `Mutex<...>`, `FlashError` com variantes, `validate_range()` com `checked_add`
- [firmware] `tasks/diagnostic.rs` reescrita (131 linhas)

---

## [1.2]

### Changed
- [firmware] `src/spi_bus.rs` reescrito — `static mut` → `Mutex<CriticalSectionRawMutex, RefCell<Option<...>>>`
- [firmware] `src/filters/deadzone.rs` reescrito — raw pointer eliminado; `apply()` retorna `(f32, bool)`
- [firmware] `src/axis/pipeline.rs` — `process()` consome flag e chama `ema.reset()` diretamente
- [firmware] `#![allow(static_mut_refs)]` removido do crate root

---

## [1.1]

### Added
- [firmware] `src/tasks/mod.rs`, `input.rs`, `hid.rs`, `diagnostic.rs` — tasks extraídas de `main.rs`
- [firmware] Feature flag `imagedef-secure-exe` no `embassy-rp` (obrigatória para boot RP2350)

### Changed
- [firmware] `src/main.rs` — reduzido de ~175 para ~120 linhas, tasks removidas

### Fixed
- [firmware] Reescrita completa de `src/sensors/mt6826.rs` — Burst Read 6-byte, CRC 3-byte, condição de magneto corrigida
- [firmware] `src/sensors/mcp23s.rs` — `read_reg()` com `blocking_transfer_in_place` (3 bytes contínuos)
- [firmware] `ANGLE_MAX = 16383` interpretado incorretamente como 14-bit — corrigido para 15-bit
- [firmware] `ANGLE_CENTER = 8192` derivado do ANGLE_MAX errado — recalculado
- [firmware] `MT6826_CMD = 0x03` — comando genérico trocado por Burst Read
- [firmware] Frame SPI de 3 bytes (Single Byte Read) — trocado por Burst
- [firmware] CRC calculado sobre 2 bytes com cobertura incorreta — corrigido
- [firmware] `check_magnet == 0x02` — interpretação invertida do STATUS — corrigida
- [firmware] `blocking_write + blocking_read` no MCP23S17 com gap no SCK — unificado em transação única
- [firmware] Tasks soltas em `main.rs` sem diretório próprio — extraídas para `src/tasks/`

---

## [1.0]

### Added
- [firmware] Build inicial — estrutura de módulos, pipeline de sinal, stack USB HID, drivers preliminares
- [firmware] `src/main.rs` — setup inicial com tasks inline (~175 linhas)
- [firmware] `src/constants.rs` — fonte única de constantes
- [firmware] `src/spi_bus.rs` — compartilhamento SPI via `static mut` + `critical_section`
- [firmware] `src/sensors/mt6826.rs` — driver MT6826S (Single Byte Read)
- [firmware] `src/sensors/mcp23s.rs` — driver MCP23S17
- [firmware] `src/calibration/data.rs` — `CalibrationData`
- [firmware] `src/filters/` — MaxJump, EMA, Deadzone, Expo, ResponseCurve
- [firmware] `src/axis/pipeline.rs` — `AxisPipeline`
- [firmware] `src/config/settings.rs` — `DeviceConfig` com load/save + CRC32
- [firmware] `src/storage/flash.rs` — primitivas de flash
- [firmware] `src/usb/` — HID descriptor, `GamepadReport`, `REPORT_SIGNAL`
