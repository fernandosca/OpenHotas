# OpenHOTAS — V1.24 Build Log

**Data:** 18/Jun/2026
**Base:** V1.23
**Tipo:** Re-análise + correções de estabilidade

---

## Resumo

V1.24 é uma versão de correção baseada em re-análise crítica do firmware V1.23.
Foca em estabilidade e robustez, sem novas features.

---

## Issues Identificados e Corrigidos

| # | Issue | Severidade | Status |
|---|-------|------------|--------|
| 1 | MaxJump não reseta ao reabilitar eixo | 🟡 Médio | ✅ Corrigido |
| 2 | `max_jump_raw` não considera range de calibração | 🔵 Baixo | ✅ Corrigido |
| 3 | Reset EMA ao mudar calibração | 🔵 Baixo | ✅ Corrigido |
| 4 | TRACK_DELTA macro para erros | ⚪ Observação | ✅ Implementado |

---

## 1. MaxJump Reset ao Reabilitar Eixo

### Problema

Eixo trava no centro após desabilitar → mover fisicamente → reabilitar.
MaxJump rejeita todas as amostras porque `last_valid` está congelado.

### Cenário de Falha

```
1. Eixo centralizado (last_valid ≈ 0.0)
2. Desabilita eixo via CLI
3. Move manípulo para extremo (+1.0)
4. Reabilita eixo
5. Delta = |1.0 - 0.0| = 1.0 > threshold
6. MaxJump rejeita → last_valid continua 0.0
7. Eixo trava no centro
```

### Correção

```rust
// axis/pipeline.rs — update_runtime_config()
if cal_changed || travel_changed || (was_disabled && cfg.enabled) {
    self.ema.reset();
    self.max_jump.reset();  // ← ADICIONADO
}
```

```rust
// filters/max_jump.rs
pub fn reset(&mut self) {
    self.initialized = false;
}
```

---

## 2. max_jump_raw com Range de Calibração

### Problema

Conversão `pa.max_jump_raw as f32 / 32767.0` assume range completo de 15-bit.
Se calibração reduzir range, threshold fica mais permissivo.

### Correção

```rust
// config/runtime.rs — from_protocol_config()
max_jump: {
    let cal_range = pa.calibration.max_raw
        .saturating_sub(pa.calibration.min_raw)
        .max(1);
    pa.max_jump_raw as f32 / cal_range as f32
}
```

---

## 3. Reset EMA ao Mudar Calibração

### Problema

Ao mudar calibração via CDC, EMA mantém histórico do sinal anterior.
Artefatos curtos aparecem na transição.

### Correção

```rust
// axis/pipeline.rs — update_runtime_config()
if cal_changed || travel_changed || (was_disabled && cfg.enabled) {
    self.ema.reset();
    self.max_jump.reset();
}
```

---

## 4. TRACK_DELTA Macro

### Problema

Código repetitivo para tracking de deltas de contadores de erro.

### Solução

```rust
macro_rules! track_delta {
    ($counter:expr, $current:expr, $prev:expr) => {
        if $current > $prev {
            $counter.fetch_add($current - $prev, Ordering::Relaxed);
            $prev = $current;
        }
    };
}
```

---

## Arquivos Alterados

| Arquivo | Mudança |
|---------|---------|
| `axis/pipeline.rs` | `max_jump.reset()` ao reabilitar |
| `filters/max_jump.rs` | Adicionado `pub fn reset()` |
| `config/runtime.rs` | `max_jump_raw` escala por range de calibração |
| `tasks/input.rs` | Macro `track_delta!` para erros |

---

## Gate de Qualidade

```
Build  : PASS
Clippy : PASS
Fmt    : PASS
```

---

*OpenHOTAS · V1.24 Build Log · Jun/2026*
