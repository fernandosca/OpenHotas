# OpenHOTAS — Riscos Técnicos Aceitos

Riscos assumidos conscientemente, com condição de segurança/mitigação. Não é changelog.

<!--
INSTRUÇÃO PARA IA — leia antes de adicionar qualquer entrada neste arquivo.

Este arquivo registra RISCOS TÉCNICOS aceitos conscientemente — trade-offs onde algo é sabidamente
não-ideal, mas foi aceito sob uma condição de segurança/mitigação específica.

Uma entrada pertence aqui quando o resumo da sessão contém algo como: "isso só é seguro se X",
"aceitei esse risco porque Y", "isso vai quebrar se Z acontecer, mas por enquanto está OK porque W",
ou uma limitação conhecida/pendente de investigação futura.
Se o resumo diz que o problema FOI corrigido nesta sessão, isso é Fixed no CHANGELOG.md, não aqui.
Se o problema ainda existe e foi só mitigado/contido, é aqui.

Ao processar o resumo de uma sessão:
1. Toda entrada nova entra em "## Pendentes / a revisitar" (nunca direto em "Resolvidos").
2. Formato: "- **[risco resumido]** (VX ou Unreleased) — mitigação: [condição de segurança]. [status: Em aberto / Confirmar / etc.]"
3. Quando uma sessão resolver um risco que já está listado em "Pendentes", MOVA a entrada pra
   "## Resolvidos" e anote em qual versão foi resolvido — não duplique a entrada.
4. NUNCA marque um risco como resolvido sem confirmação explícita no resumo da sessão.
-->

---

## Resolvidos

- **SPI via `static mut`** (V1.0) — mitigação: single-core. Resolvido em V1.2 (`Mutex<...>`).
- **Flash via `static mut`** (V1.0) — mitigação: single-core, nunca em ISR. Resolvido em V1.21 (`Mutex<...>`).
- **Deadzone raw pointer para EMA** (V1.0) — mitigação: single-core, sem concorrência no pipeline. Resolvido em V1.2.
- **`transmute` de lifetimes** (V1.0) — mitigação: inicialização única em `main.rs`. Resolvido em V1.3 (migração para `StaticCell`).
- **`install_core0_stack_guard()` não implementada** (V1.1) — API não localizada no embassy-rp 0.10. Status de resolução não confirmado no log — **verificar**.
- **Burst Read não testado em hardware físico** (V1.1/V1.2) — mitigação: checklist em `02_hardware_specs.md §5`, pendente até V1.3. Resolvido na validação em hardware de V1.3.
- **Conector frouxo no barramento dos sensores** (V1.3) — causa física identificada e corrigida.

## Pendentes / a revisitar

- **`max_jump_raw` ocultado da GUI** (V1.3) — mitigação: valor padrão ou carregado do dispositivo é preservado. Sem prazo definido.
- **Eixo Y com raros erros CRC ao mover pelo lado que atravessa o rollover** (V1.3, calibração circular) — mitigação: investigação futura (registrar `ANGLE_H`, `ANGLE_L`, `STATUS`, CRC recebido vs calculado). **Em aberto.**
- **Três MT6826S no mesmo SPI1 passaram simultaneamente para UNHEALTHY** (V1.3) — causa física (conector frouxo) identificada e corrigida; reteste com três módulos reunidos pendente. **Reteste em aberto.**
- **Validação física do fluxo de bootloader** (V1.4) — depende de confirmação em Windows com Pico 2 conectado. **Confirmar.**
- **Task não-crítica trava** (V1.3, watchdog) — mitigação: `input_task` continua rodando e alimenta watchdog normalmente. Aceito como design.
- **XIP `read_volatile` em `flash.rs` é inerentemente unsafe** (Unreleased, auditoria unsafe 2026-07-04) — mitigação: `Mutex` previne `erase`/`write` concorrentes durante leitura XIP; `validate_range` garante que offset + len está dentro dos limites da flash (2MB). O `read_volatile` é necessário para evitar que o compilador otimize leituras de MMIO. Risco residual: se a flash física for < 2MB (variante de hardware diferente), a validação usaria o valor errado. **Aceite.**
