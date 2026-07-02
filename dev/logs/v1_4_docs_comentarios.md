# Log V1.4 — Revisão de Comentários do Firmware

**Data:** 2026-07-02
**Branch:** `docs/comentarios-revisao`
**Commit:** a9a9d4b9

## O que foi feito

Revisão completa de comentários em todos os 32 arquivos `.rs` do firmware + crates de filtros (~2400 linhas de firmware + ~1500 linhas de crates).

### Regras aplicadas (12 regras)
1. ✅ Sem comentários óbvios
2. ✅ "Porquê" sobre "o quê"
3. ✅ Invariantes e suposições documentadas
4. ✅ Comentários de função/módulo com propósito, parâmetros, efeitos colaterais
5. ✅ Decisões forçadas por limitação externa (hardware, compilador, lib)
6. ✅ Cirúrgico — comentário agrega, não polui
7. ✅ Modos de falha de I/O com gaps explícitos
8. ✅ Timing/ordem de execução documentados
9. ✅ Flash e calibração: power-fall safety explicado
10. ✅ Valores empíricos com origem documentada
11. ✅ Suposições de hardware explícitas
12. ✅ Unsafe blocks com invariantes

### Arquivos críticos trabalhados
- Drivers: `mt6826.rs`, `mcp23s.rs`, `spi_bus.rs`
- Filtros: `ema.rs`, `max_jump.rs`, `deadzone.rs`, `response_curve.rs`, `calibration.rs`
- Pipeline: `pipeline.rs`, `axis/mod.rs`
- HID: `hid.rs`, `hid_gamepad.rs`, `descriptor.rs`
- Flash: `flash.rs`, `stored_config_v2.rs`
- Config: `runtime.rs`
- Tasks: `input.rs`, `cdc.rs`, `cdc_handlers.rs`
- Entry: `main.rs`

### Arquivos gerais trabalhados
- `constants.rs`, `sensors/mod.rs`, `runtime_stats.rs`, `diagnostic.rs`
- `crc32.rs`, `tuning.rs`
- Todos os `mod.rs` de re-export

## Riscos encontrados (resumo)

| Sev | Risco |
|-----|-------|
| 🔴 | SPI sem timeout — barramento pode travar |
| 🔴 | MCP MISO preso em 0x00 indetectável em runtime |
| 🟡 | MaxJump pode travar eixo em ruído contínuo |
| 🟡 | Falha de um chip MCP derruba ambos |
| 🟡 | NotInitialized propagado como sensor ausente |
| 🟡 | Swap X/Y no HID (armadilha de manutenção) |
| 🔵 | Filtros sem proteção contra NaN |
| 🔵 | Power loss entre erase e write perde config |

## Métricas
- **+930 linhas de comentários**
- **32 arquivos modificados**
- **8 riscos documentados**
- **Compilação:** firmware OK; workspace com erro pré-existente no `ssmarshal`

## Análise salva em
`dev/planos_rascunho/firmware_comentarios_revisao.md`
