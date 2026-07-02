# OpenHOTAS — Decisões Arquiteturais (ADR-lite)

Registro cronológico de decisões técnicas com motivo e status. Não é changelog — não gera entrada em release notes.

<!--
INSTRUÇÃO PARA IA — leia antes de adicionar qualquer entrada neste arquivo.

Este arquivo registra DECISÕES DE DESIGN — escolhas técnicas com trade-off e justificativa.
Não é changelog (não descreve "o que mudou"), é o "por quê" por trás de uma mudança.

Uma decisão pertence aqui, não em CHANGELOG.md, quando o resumo da sessão contém uma frase do tipo:
"decidi usar X em vez de Y porque Z", "optei por A ao invés de B", "a alternativa C foi descartada por D".
Se a sessão só descreve o que foi feito, sem justificar a escolha frente a alternativas, isso é
CHANGELOG.md, não aqui.

Ao processar o resumo de uma sessão:
1. Toda decisão nova entra em "## [Unreleased]" (crie a seção se não existir).
2. Formato: "- [decisão curta] — motivo: [porquê, incluindo alternativa descartada se houver] — status: Válida"
3. Status possíveis: "Válida" (default para decisão nova) ou "Superada em VX" (só se o resumo da
   sessão indicar explicitamente que uma decisão ANTERIOR foi revertida/substituída — nesse caso,
   edite o status da entrada antiga existente no arquivo, não crie uma entrada nova pra isso).
4. Quando o autor fizer um release, mova as decisões de [Unreleased] pra uma seção "## VX.Y.Z"
   correspondente (mesmo processo do CHANGELOG.md).
5. NUNCA invente motivo não mencionado no resumo da sessão.
-->

---

## [Unreleased]

## V1.0
- SPI compartilhado via `static mut` + `critical_section` — sound em single-core (CPSID/CPSIE). **Superada em V1.2**
- MCP23S17 com 2 chips e CS compartilhado, diferenciação via opcode. **Válida**
- USB buffers como `static mut` — exigência de lifetime `'static` do embassy-usb. **Superada em V1.3**
- Deadzone → EMA reset via raw pointer — sound em single-core, sem heap. **Superada em V1.2**
- RuntimeStats com `AtomicU32` — suficiente para single-core. **Válida**
- Flash via `static mut` + `critical_section` — erase antes de write, nunca em ISR. **Superada em V1.21**
- Tasks como closures inline em `main.rs` — simplicidade inicial. **Superada em V1.1**

## V1.1
- Burst Read adotado em vez de Single Byte Read — 1 transação vs 4, consistência atômica (latch no falling edge de CS, datasheet §8.6.8). **Válida**
- `static_mut_refs` suprimido via `#![allow(static_mut_refs)]` — transição temporária para V1.2. **Superada em V1.2**

## V1.2
- SPI via `Mutex<CriticalSectionRawMutex, RefCell<Option<...>>>` — mesma performance do `critical_section::with`, verificado pelo compilador. **Válida**
- Deadzone retorna `(f32, bool)` — pipeline consome flag e chama `ema.reset()`, elimina `*mut`. **Válida**

## V1.21
- CDC para configuração — HID fica exclusivo para gamepad. **Válida**
- `device_release` manual em BCD — conversão de string não é trivial em `no_std`. **Válida**
- Flash driver segue padrão `Mutex<...>` do `spi_bus.rs` — consistência arquitetural. **Válida**

## V1.22
- CDC para configuração via Request/Response — protocolo simples e previsível. **Válida**
- Postcard (serde) para serialização — compacto, no_std, type-safe. **Válida**
- CRC16 no frame — protege LEN + PAYLOAD contra corrupção. **Válida**
- Sem f32 no protocolo — evita NaN, Infinity, endianness issues. **Válida**
- Channel latest-wins (capacity=1) — `input_task` nunca bloqueia. **Válida**
- `cdc_task` owns config — evita Mutex global, simplifica concorrência. **Válida**

## V1.23
- Remoção de código legado V1 — substituído por StoredConfigV2. **Válida**
- Reboot com delay de 100ms — `sys_reset()` era chamado antes do buffer USB ser transmitido. **Válida**

## V1.24
- MaxJump reset ao reabilitar eixo — `last_valid` congelado impedia amostras além do threshold. **Válida**
- `max_jump_raw` escalado por range de calibração — conversão anterior assumia range completo de 15-bit. **Válida**

## V1.25
- Response Curve piecewise linear subsume expo — mais flexível (curvas assimétricas, S-curves), mais intuitivo via presets. **Válida**

## V1.26
- `Ticker::every(500µs)` — preserva meta de cadência e permite escalonamento das outras tasks. **Válida**
- Calibração rejeita eixo unhealthy — evita persistir fallback de centro como leitura válida. **Válida**
- Cleanup de sessão CDC no disconnect — evita estado órfão se PC desconectar durante calibração. **Válida**

## V1.27
- Sidebar funcional (Eixos, Botões, Curvas, Calibração, Diagnósticos) — reduzir excesso de informação, agrupar controles por contexto. **Válida**

## V1.28
- Tokens semânticos em CSS custom properties — desacopla componentes de cores específicas, permite novas identidades visuais sem alterar estrutura. **Válida**
- HUD como tema padrão — preserva comportamento visual existente. **Válida**

## V1.3
- Axis-to-Button com composição OR — botão físico e virtual podem ativar o mesmo bit HID sem desabilitar o físico. **Válida**
- `openhotas-filters` crate separado — firmware é `[[bin]]` com `test = false`; extrair lógica pura permite testes unitários no host. **Válida**
- OTP como fonte oficial de chip ID — endereços SYSINFO tinham base incorreta. **Válida**
- Pull-up em MISO após init SPI — Embassy limpa os pulls ao configurar o pad. **Válida**
- `NotPresent` representado como `FF FF FF FF` — com pull-up, sensor ausente retorna linhas em alta impedância. **Válida**
- Calibração circular — curso físico do ímã atravessa o rollover (`32767 → 0`); calibração linear anterior exigia `min < center < max`. **Válida**
- Remoção do atraso experimental de 2 µs no CS — datasheet não exige tempo morto após `CSN HIGH`. **Válida**
- Setup/hold no CSN conforme datasheet (TL 100 ns, TH 0,5 µs em 1 MHz). **Válida**
- Power-up de 5 ms — TPwrUp típico 3 ms, margem conservadora. **Válida**
- Migração `static mut` → `StaticCell` — compatibilidade com Rust 2024, elimina `unsafe` nos acessos. **Válida**
- Double-buffer com geração para power-fail safety — nunca há janela em que ambos os slots são inválidos. **Válida**
- Snapshot atômico na calibração — leituras de sensores e status devem ser consistentes no momento da captura. **Válida**
- Watchdog no `input_task` — único recovery anterior era power-cycle manual; garante reinicialização automática. **Válida**

## V1.4
- RebootToBootloader no final do enum — preserva discriminantes Postcard existentes. **Válida**
- Bootloader via ROM do RP2350 — nenhuma região adicional de flash ou bootloader próprio necessário. **Válida**
- `openhotas.uf2` removido do Git — binários devem ser distribuídos pelo CI/Releases. **Válida**
- 12 regras de comentários aplicadas — documentar invariantes, decisões forçadas por limitação externa, modos de falha de I/O, timing/ordem. **Válida**
- Finalização de calibração como cancelamento seguro — evita estado órfão na firmware se PC desconectar durante calibração. **Válida**
- Marcador de curva usa saída processada pelo firmware — evita aplicar a transformação uma segunda vez. **Válida**
