# OpenHOTAS — Extração de Changelog Técnico (V1.0 → V1.28)

> Gerado a partir dos build logs em `dev/logs/`
> Branch: `docs/changelog`

---

## V1.0

### CHANGELOG

#### Added
- [firmware] Build inicial do firmware com estrutura de módulos
- [firmware] `src/main.rs` — setup inicial com tasks inline (~175 linhas)
- [firmware] `src/constants.rs` — fonte única de constantes (valores V1.0, incorretos)
- [firmware] `src/spi_bus.rs` — compartilhamento SPI via `static mut` + `critical_section`
- [firmware] `src/sensors/mt6826.rs` — driver MT6826S com Single Byte Read (protocolo incorreto)
- [firmware] `src/sensors/mcp23s.rs` — driver MCP23S17 com `blocking_write` + `blocking_read` separados
- [firmware] `src/calibration/data.rs` — `CalibrationData` com constantes 14-bit (incorretas)
- [firmware] `src/filters/` — MaxJump, EMA, Deadzone, Expo, ResponseCurve
- [firmware] `src/axis/pipeline.rs` — `AxisPipeline` com ordem do pipeline definida
- [firmware] `src/config/settings.rs` — `DeviceConfig` com load/save + CRC32
- [firmware] `src/storage/flash.rs` — primitivas de flash (erase, write, read, crc32)
- [firmware] `src/usb/` — HID descriptor + `GamepadReport` + `REPORT_SIGNAL`

#### Fixed
- [firmware] `ANGLE_MAX = 16383` interpretado incorretamente como 14-bit [PERTENCE-A: V1.1]
- [firmware] `ANGLE_CENTER = 8192` derivado do ANGLE_MAX errado [PERTENCE-A: V1.1]
- [firmware] `MT6826_CMD = 0x03` — comando genérico em vez de Burst [PERTENCE-A: V1.1]
- [firmware] Frame SPI de 3 bytes — Single Byte Read em vez de Burst [PERTENCE-A: V1.1]
- [firmware] CRC sobre 2 bytes com cobertura incorreta [PERTENCE-A: V1.1]
- [firmware] `check_magnet == 0x02` — interpretação invertida do STATUS [PERTENCE-A: V1.1]
- [firmware] `blocking_write + blocking_read` no MCP23S17 com gap no SCK [PERTENCE-A: V1.1]
- [firmware] Tasks em `main.rs` — sem diretório `src/tasks/` [PERTENCE-A: V1.1]

### DECISIONS
- [DEC] SPI compartilhado via `static mut` + `critical_section` — motivo: sound em single-core (CPSID/CPSIE) — status: superada em V1.2
- [DEC] MCP23S17 com 2 chips e CS compartilhado — motivo: diferenciação via opcode — status: válida
- [DEC] USB buffers como `static mut` — motivo: exigência de lifetime `'static` do embassy-usb — status: superada em V1.3
- [DEC] Deadzone → EMA reset via raw pointer — motivo: sound em single-core, sem heap — status: superada em V1.2
- [DEC] RuntimeStats com `AtomicU32` — motivo: suficiente para single-core — status: válida
- [DEC] Flash via `static mut` + `critical_section` — motivo: erase antes de write, nunca em ISR — status: superada em V1.21
- [DEC] Tasks como closures inline em `main.rs` — motivo: simplicidade inicial — status: superada em V1.1

### RISKS
- [RISK] SPI via `static mut` — mitigação: single-core — superada em V1.2
- [RISK] Flash via `static mut` — mitigação: single-core, nunca em ISR — superada em V1.21
- [RISK] Deadzone raw pointer para EMA — mitigação: single-core, sem concorrência no pipeline — superada em V1.2
- [RISK] `transmute` de lifetimes — mitigação: inicialização única em `main.rs`

---

## V1.1

### CHANGELOG

#### Added
- [firmware] `src/tasks/mod.rs` — módulo de tasks (`pub mod input; pub mod hid; pub mod diagnostic;`)
- [firmware] `src/tasks/input.rs` — `input_task` extraída de `main.rs`
- [firmware] `src/tasks/hid.rs` — `usb_task` + `hid_task` extraídas de `main.rs`
- [firmware] `src/tasks/diagnostic.rs` — `diagnostic_task` extraída de `main.rs`
- [firmware] Feature flag `imagedef-secure-exe` no `embassy-rp` (obrigatória para boot RP2350)

#### Changed
- [firmware] Reescrita completa de `src/sensors/mt6826.rs` — Burst Read 6-byte, CRC 3-byte, condição magneto corrigida
- [firmware] `src/sensors/mcp23s.rs` — `read_reg()` com `blocking_transfer_in_place` (3 bytes contínuos)
- [firmware] `src/constants.rs` — 3 constantes corrigidas: `ANGLE_MAX`, `ANGLE_CENTER`, `CMD_READ_ANGLE`
- [firmware] `src/main.rs` — reduzido de ~175 para ~120 linhas, tasks removidas
- [firmware] `src/usb/hid_gamepad.rs` — `usb_task` e `hid_task` removidas
- [firmware] `src/context/_context.rs` — valores 15-bit, protocolo Burst Read
- [firmware] `src/context/_pinout.rs` — protocolo e constantes atualizados

### DECISIONS
- [DEC] Burst Read adotado (não Single Byte Read) — motivo: 1 transação vs 4, consistência atômica (latch no falling edge de CS, §8.6.8) — status: válida
- [DEC] `static_mut_refs` suprimido via `#![allow(static_mut_refs)]` — motivo: transição para V1.2 — status: superada em V1.2

### RISKS
- [RISK] SPI e Flash via `static mut` — mitigação: revisar se SMP for adotado
- [RISK] Deadzone usando raw pointer para EMA — mitigação: sound no contexto atual
- [RISK] Burst Read não testado em hardware físico — mitigação: checklist em `02_hardware_specs.md §5` — **Pendente até V1.3**
- [RISK] `install_core0_stack_guard()` não implementada — mitigação: API não localizada no embassy-rp 0.10

---

## V1.2

### CHANGELOG

#### Changed
- [firmware] `src/spi_bus.rs` reescrito — `static mut` → `Mutex<CriticalSectionRawMutex, RefCell<Option<...>>>`
- [firmware] `src/filters/deadzone.rs` reescrito — raw pointer eliminado; `apply()` retorna `(f32, bool)`
- [firmware] `src/axis/pipeline.rs` editado — `process()` consome flag booleana e chama `ema.reset()` diretamente
- [firmware] `src/main.rs` — `#![allow(static_mut_refs)]` removido do crate root

### DECISIONS
- [DEC] SPI via `embassy_sync::Mutex<CriticalSectionRawMutex, RefCell<Option<...>>>` — motivo: mesma performance do `critical_section::with`, verificado pelo compilador — status: válida
- [DEC] Deadzone retorna `(f32, bool)` — motivo: owner (pipeline) consome flag e chama `ema.reset()`, eliminando `*mut` — status: válida

### RISKS
- [RISK] Flash via `static mut` — mitigação: nunca chamar de ISR — candidato a refatoração em V3
- [RISK] Burst Read MT6826S não testado em hardware físico — mitigação: checklist em `02_hardware_specs.md §5` — **Pendente, ação obrigatória antes de V2**

---

## V1.21

### CHANGELOG

#### Added
- [firmware] Sistema de versionamento — três camadas: Cargo.toml (SemVer), build.rs (git hash), USB descriptor (BCD)
- [firmware] `src/constants.rs` — `FIRMWARE_VERSION` e `FIRMWARE_GIT_HASH`
- [firmware] `build.rs` — injeção de `GIT_HASH` via `env!("GIT_HASH")` com `rerun-if-changed`
- [firmware] CDC ACM — canal auxiliar de debug com `CdcAcmClass`, banner, telemetria a cada 5s, alerta `WARN:max_cycle`
- [firmware] `src/diagnostics/runtime_stats.rs` — atomics tornados `pub`

#### Changed
- [firmware] `src/storage/flash.rs` reescrita — `static mut FLASH_INSTANCE` → `Mutex<...>`, `FlashError` com `OutOfBounds`/`NotInitialized`, `validate_range()` com `checked_add`
- [firmware] `src/tasks/diagnostic.rs` reescrita — 131 linhas, reescrita completa

### DECISIONS
- [DEC] CDC para configuração — motivo: HID fica exclusivo para gamepad — status: válida
- [DEC] `device_release` manual em BCD — motivo: conversão de string não é trivial em `no_std` — status: válida
- [DEC] Flash driver segue padrão `Mutex<...>` do `spi_bus.rs` — motivo: consistência arquitetural — status: válida

---

## V1.22

### CHANGELOG

#### Added
- [firmware] Configuration Protocol — configuração e calibração via USB CDC com protocolo binário request/response
- [firmware] `src/tasks/cdc.rs` — protocolo binário request/response
- [firmware] `src/tasks/cdc_handlers.rs` — handlers de leitura/escrita/calibração
- [firmware] `src/config/runtime.rs` — `RuntimeConfig`, `CONFIG_SIGNAL` (Channel capacity=1)
- [firmware] `src/config/stored_config_v2.rs` — persistência flash com layout V2 (MAGIC + VERSION + PAYLOAD + CRC32)
- [firmware] RuntimeConfig com validação de ranges (deadzone, ema, max_jump, travel, calibration ordering)
- [crates] `crates/openhotas-protocol/` — 8 arquivos: `lib.rs`, `config.rs`, `request.rs`, `response.rs`, `diagnostics.rs`, `error.rs`, `frame.rs`, `version.rs`
- [crates] Frame format: SOF (0xAA 0x55) + LEN (u16 BE) + PAYLOAD (postcard) + CRC16-CCITT (u16 BE)

#### Changed
- [firmware] `src/usb/descriptor.rs` — Report ID desnecessário removido, Logical Minimum alterado de +1 para -32767 (i16 assinado)
- [firmware] `src/tasks/diagnostic.rs` — desacoplada do CDC, voltou a logar apenas via `defmt`

### DECISIONS
- [DEC] CDC para configuração (Request/Response) — motivo: HID fica exclusivo para gamepad, protocolo simples e previsível — status: válida
- [DEC] Postcard (serde) para serialização — motivo: compacto, no_std, type-safe — status: válida
- [DEC] CRC16 no frame — motivo: protege LEN + PAYLOAD contra corrupção — status: válida
- [DEC] Sem f32 no protocolo — motivo: evita NaN, Infinity, endianness issues — status: válida
- [DEC] Channel latest-wins (capacity=1) — motivo: input_task nunca bloqueia — status: válida
- [DEC] cdc_task owns config — motivo: evita Mutex global, simplifica concorrência — status: válida

---

## V1.23

### CHANGELOG

#### Added
- [firmware] Helper `signal_latest_config()` — se canal cheio, drena antiga e retenta (latest-wins)
- [cli] `openhotas-cli` — CLI funcional com 13 comandos: `info`, `get-config`, `set-axis`, `save`, `calibrate`, `raw-axes`, `processed-axes`, `buttons`, `stats`, `errors`, `sensor-status`, `load-defaults`, `reboot`, `factory-reset`

#### Changed
- [firmware] Reboot com Ack + delay 100ms (`Timer::after_millis(100).await` antes de `SCB::sys_reset()`)

#### Fixed
- [firmware] `CaptureCalibrationPoint` rejeita `raw == 0` — corrigido
- [firmware] `expo_i16` negativo ignorado — corrigido
- [firmware] `save_config` 4KB na stack — corrigido (buffer otimizado)
- [firmware] `Channel::try_send` silencioso quando canal cheio — corrigido (latest-wins)
- [firmware] CRC protocolo vs CRC sensor — separados

#### Removed
- [firmware] `cal_store.rs` — substituído por `stored_config_v2.rs`
- [firmware] `settings.rs` — substituído por `stored_config_v2.rs`
- [firmware] `sensor_health.rs` — substituído por `runtime_stats.rs`
- [firmware] Constantes `CALIB_OFFSET`, `CONFIG_OFFSET`, `MAGIC_*` — layout V1 obsoleto

### DECISIONS
- [DEC] Remoção de código legado V1 — motivo: substituído por StoredConfigV2 — status: válida
- [DEC] Reboot com delay 100ms — motivo: `sys_reset()` chamado antes do buffer USB ser transmitido — status: válida

---

## V1.24

### CHANGELOG

#### Fixed
- [firmware] MaxJump não reseta ao reabilitar eixo — `last_valid` congelado causava trava no centro
- [firmware] `max_jump_raw` não considera range de calibração — threshold ficava mais permissivo com range reduzido
- [firmware] EMA não reseta ao mudar calibração — artefatos curtos na transição

#### Changed
- [firmware] Macro `track_delta!` para tracking de deltas de contadores de erro

### DECISIONS
- [DEC] MaxJump reset ao reabilitar — motivo: `last_valid` congelado impedia amostras além do threshold — status: válida
- [DEC] `max_jump_raw` escala por range de calibração — motivo: conversão `pa.max_jump_raw as f32 / 32767.0` assumia range completo de 15-bit — status: válida

---

## V1.25

### CHANGELOG

#### Added
- [firmware] Response Curve piecewise linear com 5 pontos de controle (P0=(-1,-1), P1, P2=(0,0), P3, P4=(1,1))
- [crates] `CurvePoint`, `ResponseCurveData` adicionados ao protocol crate
- [gui] `CurvePage.tsx` — piecewise linear com presets (Linear, Suave, Centro, S-curve)

#### Changed
- [firmware] `filters/response_curve.rs` reescrito — piecewise linear em vez de curva cúbica simples
- [firmware] `axis/mod.rs` — `expo` substituído por `response_p1`, `response_p3`
- [firmware] `axis/pipeline.rs` — expo removido, response integrado
- [firmware] `config/runtime.rs` — conversão `ResponseCurveData` → `(f32, f32)`
- [firmware] `constants.rs` — removido `DEFAULT_EXPO`
- [protocol] `expo_i16` removido de `AxisConfig`, `PROTOCOL_VERSION_MAJOR` 1 → 2
- [gui] `protocol.ts` — tipos atualizados, `CurveEditor.tsx` redesenhado

#### Fixed
- [firmware] Protocol version mismatch — `protocol.ts` `1` → `2`
- [firmware] CRC8 overflow — `wrapping_shl(1)`
- [firmware] Deadzone div/zero — guard `threshold >= 1.0`
- [firmware] SPI bus `expect()` — `Result<R, SpiBusError>`
- [firmware] `read_flash` init check — verificação antes de XIP
- [firmware] `GIT_HASH` sem fallback — `option_env!` + `"unknown"`

#### Removed
- [firmware] `filters/expo.rs` — subsumado por response curve
- [firmware] `REPORT_ID_CONFIG` de `constants.rs`
- [firmware] `#[allow(dead_code)]` de `axis/mod.rs` em `healthy`

### DECISIONS
- [DEC] Response Curve piecewise linear subsume expo — motivo: mais flexível (curvas assimétricas, S-curves), mais intuitivo (presets no configurador) — status: válida

---

## V1.26

### CHANGELOG

#### Added
- [gui] Nova tela de botões — `ButtonsPage.tsx` com grid de 32 botões, lista ativa, masks, debounce
- [cli] Handshake obrigatório — `OpenHotasTransport::connect_to()` envia `GetInfo` e valida `protocol_major`, `axis_count`, `button_count`
- [cli] Auto-detect por identidade — `connect()` tenta cada porta e valida handshake `GetInfo`

#### Changed
- [gui] `App.tsx` — navegação reorganizada
- [gui] `Dashboard.tsx` — consolidado como tela `Eixos`
- [gui] `CurvePage.tsx` — presets simplificados (sem edição de pontos)
- [gui] `Calibration.tsx` — layout consolidado em único card
- [gui] `Diagnostics.tsx` — cards unificados
- [firmware] `tasks/input.rs` — `Ticker::every(Duration::from_micros(500))` cadencia o loop
- [firmware] `tasks/cdc_handlers.rs` — `CaptureCalibrationPoint` rejeita captura se eixo unhealthy
- [firmware] `tasks/cdc.rs` — sessão de calibração limpa ao desconectar CDC

### DECISIONS
- [DEC] `Ticker::every(500µs)` — motivo: preserva meta de 500µs e permite escalonamento das outras tasks — status: válida
- [DEC] Calibração rejeita unhealthy — motivo: prevê persistir fallback de centro como leitura válida — status: válida
- [DEC] Cleanup de sessão CDC no disconnect — motivo: prevê estado órfão se PC desconectar durante calibração — status: válida

---

## V1.27

### CHANGELOG

#### Changed
- [gui] Tela `Configuração` removida da navegação — controles transferidos para `Eixos` e `Botões`
- [gui] `Dashboard.tsx` — controles de eixo consolidados em card único (Habilitado, Invertido, Reset EMA, Filtros, Travel limits)
- [gui] `ButtonsPage.tsx` — card de configuração movido para esta tela (grid 32 botões, lista ativa, masks, debounce)
- [gui] `CurvePage.tsx` — gráfico simplificado para visualização (sem edição de pontos), presets: Linear, Suave, Centro, S-curve
- [gui] `Calibration.tsx` — consolidada em único card, coluna lateral removida, status e instruções centralizados
- [gui] `Diagnostics.tsx` — `Runtime stats` no topo, `Atualizar tudo` como texto clicável, `Ações do dispositivo` na parte inferior

#### Removed
- [gui] Arquivos antigos da tela `Configuração`
- [gui] Story da tela `Configuração`
- [gui] Referências de edição visual por Puck

### DECISIONS
- [DEC] Sidebar funcional (Eixos, Botões, Curvas, Calibração, Diagnósticos) — motivo: reduzir excesso de informação, agrupar controles por contexto — status: válida

---

## V1.28

### CHANGELOG

#### Added
- [gui] Infraestrutura de temas — `ThemeProvider` com `data-theme` no elemento raiz, persistência em `localStorage`, fallback `HUD`
- [gui] `gui/src/theme/themes.ts` — registro tipado dos temas disponíveis e do tema padrão
- [gui] Tokens semânticos — contrato de cores em `gui/src/theme/theme.css` (fundo, borda, texto, accent, estados, eixos, canvas)
- [gui] Tema `HUD` — formaliza visual original (superfícies azul-escuras, accent ciano)
- [gui] Tema `Glass Cockpit` — paleta oliva-escuros, textos dessaturados, accent azul-névoa
- [gui] Seletor de aparência na tela `Configurações`

#### Changed
- [gui] `gui/src/theme/colors.ts` — valores estáticos substituídos por resolução dos tokens ativos via `getComputedStyle`
- [gui] `gui/tailwind.config.ts` — Tailwind consome variáveis do contrato de temas
- [gui] `gui/src/index.css` — slots shadcn vinculados ao tema ativo
- [gui] Canvas e gráficos redesenham quando tema muda (HUD vetorial, editor de curvas)
- [gui] Storybook — preview global com `ThemeProvider`, stories migradas para tokens semânticos

### DECISIONS
- [DEC] Tokens semânticos em CSS custom properties — motivo: desacoplar componentes de cores específicas, permitir novas identidades visuais sem alterar estrutura — status: válida
- [DEC] HUD como tema padrão — motivo: preservar comportamento visual existente — status: válida

---

## V1.3

### CHANGELOG

#### Added
- [firmware] Axis-to-Button — mapeia posição de eixo para botão virtual no report HID (threshold em permille, direção, button_index)
- [firmware] Center Offset — ajuste fino do ponto zero em permille (-200..200) sem recalibrar
- [firmware] Validação de colisão — botões virtuais não colidem entre si; botão virtual e físico compõem por OR no mesmo bit HID
- [firmware] `openhotas-filters` crate — extração de lógica pura (EMA, Deadzone, MaxJump, ResponseCurve, Calibration, CRC32) com 29 testes unitários
- [crates] `AxisDirection`, `AxisToButtonConfig` em `openhotas-protocol/src/config.rs`
- [crates] `center_offset_permille` em `AxisConfig`
- [gui] Controles visuais para `center_offset_permille` e `axis_to_button` em `Dashboard.tsx`
- [cli] `--center-offset`, `--axis-to-button` em `commands.rs`
- [firmware] `serial USB contract` — formato `"OH{:016X}"` (18 bytes) documentado em `04_software_contracts.md §8.1`

#### Changed
- [firmware] `openhotas-protocol` versionado como `1.3.0`
- [protocol] `PROTOCOL_VERSION_MAJOR`: 2 → 3
- [gui] Versionada como `1.3.0` em `package.json`, `tauri.conf.json`, `src-tauri/Cargo.toml`
- [cli] Versionada como `1.3.0`
- [firmware] SERIAL_STR refactor — raw pointer writes byte-a-byte → buffer local + `copy_nonoverlapping`, `unsafe` aninhado (3 blocos) → 1 único `unsafe` block

#### Fixed
- [firmware] CHIP_ID — endereços `0x00010040`/`0x00010044` incorretos (base `0x00010000` não é SYSINFO `0x40000000`) — substituído por `otp::get_chipid()`
- [firmware] Serial USB poderia ser igual em todas as placas — corrigido com OTP

#### Removed
- [firmware] Constantes `CHIP_ID_HI`/`CHIP_ID_LO` de `main.rs`

### DECISIONS
- [DEC] Axis-to-Button com composição OR — motivo: botão físico e virtual podem ativar o mesmo bit HID sem desabilitar o botão físico — status: válida
- [DEC] `openhotas-filters` crate separado — motivo: firmware é `[[bin]]` com `test = false`; extrair lógica pura permite testes unitários no host sem dependências HAL/embassy — status: válida
- [DEC] OTP como fonte oficial de chip ID — motivo: endereços SYSINFO tinham base incorreta — status: válida

### RISKS
- [RISK] `max_jump_raw` ocultado da GUI — mitigação: valor padrão ou carregado do dispositivo é preservado

---

## V1.3 (cont.) — Validação em Hardware (Raspberry Pi Pico 2 / RP2350A)

### CHANGELOG

#### Fixed
- [firmware] `memory.x` com layout de RP2040 (BOOT2 de 256 bytes em `0x10000000`) — removido; FLASH alterada para `ORIGIN = 0x10000000`; `.start_block` inserido após `.vector_table`
- [firmware] MISO flutuante do MT6826S produzia frames zerados com CRC zero falsamente positivos — pull-up interno habilitado em `SPI0_MISO` e `SPI1_MISO`
- [firmware] Escritas SPI no MCP23S17 retornavam sucesso sem chip conectado — inicialização lê de volta `IOCON`, `IODIR` e `GPPU` nos dois endereços e exige valores configurados
- [firmware] Resposta `00 00 00 00` classificada como sensor ausente (apesar de ser legalmente ângulo zero com CRC zero) — com pull-up no MISO, ausente passa a ser `FF FF FF FF`
- [firmware] Novo erro interno `SensorError::NotPresent` — diferencia ausência física de falha de transporte

#### Changed
- [firmware] Conversor `elf2uf2-rs` removido — fluxo oficial passou a usar `picotool uf2 convert ... --abs-block`
- [firmware] Frequência SPI final: 1 MHz

### DECISIONS
- [DEC] Pull-up em MISO após init SPI — motivo: Embassy limpa os pulls ao configurar o pad — status: válida
- [DEC] `NotPresent` como `FF FF FF FF` — motivo: com pull-up, sensor ausente retorna todas as linhas em alta impedância — status: válida

### RISKS
- [RISK] Conector frouxo no barramento dos sensores — mitigação: causa física identificada e corrigida

---

## V1.3 (cont.) — Calibração Circular dos Eixos

### CHANGELOG

#### Added
- [firmware] Calibração circular de 15 bits — cada amostra e extremo representados pela menor distância assinada em relação ao centro: `delta = ((raw - center + 16384) mod 32768) - 16384`
- [firmware] Testes automatizados para passagem pelo zero, centro/limites corretos, rejeição de extremos no mesmo lado
- [crates] 32/32 testes em `openhotas-filters`, 8/8 em `openhotas-protocol`

#### Changed
- [firmware] `max_jump_raw` passa a ser escalado pelo curso circular calibrado
- [firmware] Ordem numérica dos pontos brutos deixou de ser relevante

### DECISIONS
- [DEC] Calibração circular — motivo: curso físico do ímã atravessou o rollover (`32767 → 0`), calibração linear anterior exigia `min_raw < center_raw < max_raw` — status: válida

### RISKS
- [RISK] Eixo Y com raros erros CRC ao mover pelo lado que atravessa o rollover — mitigação: investigação futura (registrar `ANGLE_H`, `ANGLE_L`, `STATUS`, CRC recebido vs calculado)

---

## V1.3 (cont.) — Proteção de CS no Barramento MT6826S

### CHANGELOG

#### Changed
- [firmware] Margem entre sensores (2 µs após `CSN HIGH`) testada e posteriormente removida — datasheet não exige tempo morto depois de `CSN HIGH`

### DECISIONS
- [DEC] Remoção do atraso experimental de 2 µs — motivo: datasheet indica MISO retorna para alta impedância na subida de CSN, sem espera em escala de microssegundos — status: válida

### RISKS
- [RISK] Três MT6826S no mesmo SPI1 passaram simultaneamente para UNHEALTHY — mitigação: causa física (conector frouxo) identificada e corrigida; reteste com três módulos reunidos pendente

---

## V1.3 (cont.) — Revisão Final MT6826S e Toolchain RP2350

### CHANGELOG

#### Changed
- [firmware] Driver MT6826S — setup de 1 µs entre `CSN LOW` e primeiro clock (TL mínimo: 100 ns)
- [firmware] Driver MT6826S — hold de 1 µs antes de `CSN HIGH` (TH mínimo em 1 MHz: 0,5 µs)
- [firmware] `input_task` — aguarda 5 ms antes da primeira leitura (TPwrUp típico: 3 ms)
- [firmware] Instrução temporária `sensor-frames` removida após causa física ser identificada
- [firmware] Revisão do linker RP2350 — removido `rp2350-link.x` privado, restaurado `link.x` oficial do `cortex-m-rt 0.7.5`, seção `.start_block` mantida em `memory.x`

### DECISIONS
- [DEC] Setup/hold no CSN — motivo: datasheet exige TL (100 ns) e TH (0,5 µs em 1 MHz) — status: válida
- [DEC] Power-up de 5 ms — motivo: TPwrUp típico 3 ms, margem conservadora — status: válida

---

## V1.3 (cont.) — Validação em Hardware

### CHANGELOG

#### Fixed
- [firmware] CS pins dos encoders X e Y invertidos fisicamente no hardware de teste — solução via `set-axis --axis x --invert true` pela CLI

### DECISIONS
- [DEC] Validação em hardware V1.3 — resultado: firmware operou sem erros, crashes ou resets inesperados; `sensor-status` zero erros; problema encontrado foi mal contato na fiação

---

## V1.3 (cont.) — Crate `openhotas-filters`

### CHANGELOG

#### Added
- [crates] `crates/openhotas-filters/` — crate library com `#![no_std]`, dependência apenas de `libm`
- [crates] 29 testes: EMA (6), Deadzone (5), MaxJump (4), ResponseCurve (4), Calibration (6), CRC32 (4)

#### Changed
- [firmware] `filters/mod.rs` → re-exporta tipos do novo crate
- [firmware] `calibration/mod.rs` → re-exporta `CalibrationData`, `Calibration`
- [firmware] `storage/flash.rs` → re-exporta `crc32`

---

## V1.3 (cont.) — Correções de Robustez

### CHANGELOG

#### Added
- [firmware] Double-buffer com geração + CRC32 para power-fail safety em `config/stored_config_v2.rs` — boot lê ambos slots, usa o de maior geração; save escreve no slot inativo com geração = atual + 1
- [firmware] Compile-time assertion para `frame_buf` em `tasks/cdc.rs` — `const _: () = assert!(4 + MAX_PAYLOAD_SIZE + 2 <= 300, ...);`
- [firmware] Snapshot atômico com `critical_section::with` na calibração — leituras de `SENSOR_UNHEALTHY` e `RAW_AXIS_*` consistentes no momento da captura
- [firmware] `reset_max_cycle()` — retorna pico da janela anterior e reseta contador, chamada pelo `diagnostic_task` a cada 5s
- [firmware] Safety comment em `axis_to_i16` em `usb/hid_gamepad.rs`
- [firmware] Proteção contra truncation em `as_micros()` em `tasks/input.rs`

#### Changed
- [firmware] `sensors/mt6826.rs` — constante `MT6826_CMD_READ_ANGLE` em vez de magic number `0xA0`
- [firmware] `constants.rs` — constantes `STORED_V2_SLOT_A`/`STORED_V2_SLOT_B`

#### Removed
- [firmware] Constante morta `REPORT_ID_GAMEPAD` de `constants.rs`

### DECISIONS
- [DEC] Double-buffer com geração — motivo: power-fail safety, nunca há janela onde ambos slots são inválidos — status: válida
- [DEC] Snapshot atômico na calibração — motivo: leituras de sensores e status devem ser consistentes no momento da captura — status: válida

### RISKS
- [RISK] I-1 (migração `static mut` → `StaticCell`) bloqueado — mitigação: requer adição de crate `static-cell` e refatoração dos 7 `static mut` em `main.rs` — resolvido em V1.3 (seção seguinte)

---

## V1.3 (cont.) — Migração `static mut` → `StaticCell`

### CHANGELOG

#### Changed
- [firmware] `main.rs` — 7 `static mut` migrados para `StaticCell` (crate `static_cell = "2.1.1"`): `DD`, `CD`, `BD`, `CB`, `HS`, `CDC_STATE`, `SERIAL_STR`
- [firmware] Padrões `unsafe { addr_of_mut!(...) }` e `transmute` substituídos por `StaticCell::init()`
- [firmware] `unsafe fn chip_id_serial_static()` → `fn chip_id_serial_static()` (safe)
- [firmware] `copy_nonoverlapping` + `addr_of_mut!(SERIAL_STR)` → `SERIAL_STR.init([0u8; 18])` + write direto
- [firmware] `#[allow(static_mut_refs)]` (3 ocorrências) removido

#### Added
- [firmware] Dependência `static_cell = "2.1.1"` em `Cargo.toml`

### DECISIONS
- [DEC] Migração `static mut` → `StaticCell` — motivo: compatibilidade com Rust 2024, eliminação de `unsafe` nos acessos — status: válida

---

## V1.3 (cont.) — Hardware Watchdog

### CHANGELOG

#### Added
- [firmware] Watchdog de hardware via `embassy_rp::watchdog::Watchdog` — timeout de 2000 ms, alimentado a cada ciclo (500 µs) em `input_task`
- [firmware] Constante `WATCHDOG_TIMEOUT_MS = 2000`

#### Changed
- [firmware] `Cargo.toml` — `embassy-time` atualizado de 0.4 → 0.5 (compatibilidade com `Duration` do `embassy-rp 0.10`)
- [firmware] `tasks/input.rs` — `wdt.feed(Duration::from_millis(WATCHDOG_TIMEOUT_MS))` a cada iteração do loop

### DECISIONS
- [DEC] Watchdog no `input_task` — motivo: se firmware travar, único recovery era power-cycle manual; watchdog garante reinicialização automática — status: válida

### RISKS
- [RISK] Task não-crítica trava — mitigação: `input_task` continua rodando e alimenta watchdog normalmente

---

## V1.4

### CHANGELOG

#### Added
- [firmware] `Request::RebootToBootloader` — novo comando aditivo ao final do enum para preservar discriminantes Postcard existentes
- [firmware] Estado `PendingReset` — distingue reboot normal e entrada no bootloader
- [firmware] `reset_to_usb_boot(0, 0)` via `embassy_rp::rom_data` — coloca RP2350 no bootloader USB da ROM
- [cli] `openhotas-cli bootloader` — valida separadamente a transição para o volume RPI-RP2
- [gui] Seletor nativo limitado a arquivos `.uf2`
- [gui] Confirmação explícita antes do reboot e da gravação
- [gui] Validação backend: extensão, tamanho, alinhamento de 512 bytes e assinaturas de blocos UF2
- [gui] Descoberta de volume para Windows, Linux e macOS (timeout 15 s, polling 200 ms)
- [gui] Cópia como `openhotas.uf2` com comparação de tamanho validado
- [gui] Tela com seleção, progresso, conclusão e erro
- [gui] Plugin oficial de diálogo do Tauri + permissões restritas

#### Changed
- [protocol] Versão binária: `3.1` (novo comando aditivo)
- [gui] Logo sidebar substituída por imagem provisória de aviação `gui/src/assets/512x512px.png`
- [gui] Metadados `Zone.Identifier` adicionados ao `.gitignore`
- [gui] Bundle usa conjunto de ícones desktop gerado a partir do PNG RGBA de 512 px
- [gui] `Cargo.lock` e `gui/package-lock.json` atualizados com plugin de diálogo Tauri

#### Removed
- [firmware] `firmware/openhotas.uf2` retirado do índice Git — novos binários distribuídos pelo GitHub Actions e Releases
- [gui] Regras globais `*.elf` e `*.uf2` adicionadas ao `.gitignore` da raiz

### DECISIONS
- [DEC] RebootToBootloader no final do enum — motivo: preservar discriminantes Postcard existentes — status: válida
- [DEC] Bootloader via ROM do RP2350 — motivo: nenhuma região adicional de flash ou bootloader próprio necessário — status: válida
- [DEC] `openhotas.uf2` removido do Git — motivo: binários devem ser distribuídos pelo CI/Releases — status: válida

### RISKS
- [RISK] Validação física pendente — mitigação: fluxo completo depende de hardware, deve ser confirmado no Windows com Pico 2 conectado

---

## V1.4 (cont.) — Revisão de Comentários do Firmware

### CHANGELOG

#### Changed
- [firmware] Revisão completa de comentários em todos os 32 arquivos `.rs` do firmware + crates (~2400 + ~1500 linhas) — +930 linhas de comentários

#### Added
- [firmware] Documentação de 8 riscos em comentários: SPI sem timeout, MCP MISO preso em 0x00, MaxJump pode travar em ruído contínuo, falha de chip MCP derruba ambos, NotInitialized propagado como ausente, swap X/Y no HID, filtros sem proteção NaN, power loss entre erase e write

### DECISIONS
- [DEC] 12 regras de comentários aplicadas — motivo: documentar invariantes, decisões forçadas por limitação externa, modos de falha de I/O, timing/ordem — status: válida

---

## V1.4 (cont.) — Windows GUI Fixes

### CHANGELOG

#### Changed
- [gui] Removida moldura nativa da janela — controles duplicados de minimizar/fechar junto à barra personalizada
- [gui] Barra superior personalizada — passou a permitir arraste da janela
- [gui] Permissões Tauri liberadas para fechar, minimizar e iniciar arraste

#### Added
- [gui] Atualização imediata de estado após conectar/desconectar (sem depender de operação de gravação)
- [gui] Polling de eixos e estatísticas disparado após conexão
- [gui] Recarga de configuração ativa ao conectar e após calibração
- [gui] Marcador em tempo real no gráfico da curva (saída processada pelo firmware)
- [gui] Compensação de eixos invertidos na visualização do marcador

#### Fixed
- [gui] Erros de calibração — descartam sessão pendente no firmware usando finalização como cancelamento seguro; interface retorna ao estado inicial
- [gui] Salvamento de curvas — deixou de sobrescrever calibração recente com cópia local antiga dos limites
- [gui] Grade de botões — exibe índices `0..31` alinhados ao protocolo/firmware/campo de configuração

### DECISIONS
- [DEC] Finalização como cancelamento seguro — motivo: prevê estado órfão na firmware se PC desconectar durante calibração — status: válida
- [DEC] Marcador de curva usa saída processada — motivo: evita aplicar a transformação uma segunda vez — status: válida
