# Macros (Sequenciador de Botões)

> Feature para futura implementação. Não é um plano de implementação detalhado.

---

## Visão Geral

Sequência de botões executada com 1 pressionamento. Firmware armazena macros na flash e executa localmente.

**Cenário principal:** Startup sequence, weapon arm, landing gear.

---

## Mecânica

```
Botão "startup" pressionado →
  Step 1: Botão 5 ON
  Step 2: Espera 500ms
  Step 3: Botão 5 OFF, Botão 8 ON
  Step 4: Espera 300ms
  Step 5: Botão 8 OFF
```

---

## Estrutura de Dados

```rust
struct MacroStep {
    button_mask: u32,    // quais botões ativar
    delay_ms: u16,       // espera antes do próximo step
}

struct Macro {
    steps: [MacroStep; 8],
    step_count: u8,
}
```

---

## Parâmetros

| Parâmetro | Default | Range | Descrição |
|---|---|---|---|
| `trigger_button` | 0 | 0–31 | Botão que ativa a macro |
| `step_count` | 0 | 0–8 | Número de passos |
| `steps[0..7]` | — | MacroStep | Sequência de passos |

---

## Armazenamento

```
Flash: 1 setor dedicado (4KB)
  → ~50 macros de 8 passos
  → Escrita via GUI, não em tempo real
  → Leitura no input_task
```

---

## Arquivos Envolvidos

| Arquivo | Ação | Descrição |
|---|---|---|
| `macros/mod.rs` | Criar | Engine de macros |
| `macros/sequencer.rs` | Criar | State machine de replay |
| `macros/storage.rs` | Criar | Flash persistence |
| `tasks/input.rs` | Modificar | Chamar sequencer |
| `config/settings.rs` | Modificar | Referência a macros |
| `constants.rs` | Modificar | Defaults |

---

## Custo Estimado

| Recurso | Consumo |
|---|---|
| Flash código | ~200 bytes |
| Flash dados | ~4KB (1 setor) |
| RAM | ~80 bytes (1 macro em execução) |
| CPU | ~10µs por step |
| Linhas de código | ~150 |

---

## Limitações

1. **Máximo 8 passos por macro**
2. **Sem nesting** — macro não chama outra macro
3. **Sem condicionais** — sequência linear
4. **Flash writes limitados** — ~100K ciclos

---

## Casos de Uso

| Macro | Passos | Utilidade |
|---|---|---|
| Startup sequence | 4-6 | Alto |
| Weapon arm | 2-3 | Alto |
| Landing gear | 2 | Médio |
| Radio preset | 3-4 | Médio |

---

## Decisão de Design: Firmware vs Software

| Critério | Firmware | Software |
|---|---|---|
| Complexidade | MÉDIA | ALTA |
| Linhas | ~150 | ~500-1000 |
| Latência | ~1ms | ~5-20ms |
| Standalone | ✅ | ❌ |
| Flexibilidade | Limitado | Ilimitado |

**Recomendação:** Firmware primeiro (curto prazo), software depois (médio prazo).

---

*OpenHOTAS · Plan · Jun/2026*
