# OpenHOTAS — Features Potenciais V1.4+

> Features de baixa complexidade que podem ser implementadas no futuro.

---

## 1. Button Long Press

### Conceito

Detectar press longo (>500ms) e gerar um botão virtual separado.

```
Botão físico 0 curto  → Botão 0 no HID
Botão físico 0 longo  → Botão 32 no HID (se disponível)
```

### Implementação

**Protocol:** Adicionar `long_press_threshold_ms: u16` ao `ButtonConfig`

**Firmware:**
- Armazenar timestamp de cada botão
- Verificar duração no ciclo do input_task
- Setar bit adicional se press > threshold

**Complexidade:** ⭐⭐

---

## 2. Button Toggle

### Conceito

Modo toggle para botões: press liga, press desliga.

```
Botão 0 modo normal:   press = 1, release = 0
Botão 0 modo toggle:   press = inverte estado, release = mantém
```

### Implementação

**Protocol:** Adicionar `toggle_mask: u32` ao `ButtonConfig`

**Firmware:**
-.bits no `toggle_mask` = botões em modo toggle
- Armazenar estado toggle por botão
- Inverter estado na borda de descida

**Complexidade:** ⭐⭐

---

## 3. Sensitivity Per Axis

### Conceito

Multiplicador de sensibilidade por eixo (50-200%).

```
Sensitivity = 80% → range [-0.8, +0.8] em vez de [-1.0, +1.0]
Sensitivity = 120% → range [-1.0, +1.0] com saturação
```

### Implementação

**Protocol:** Adicionar `sensitivity_permille: u16` ao `AxisConfig`

**Firmware:**
- Aplicar após center offset, antes de travel limits
- `output = input * (sensitivity / 1000.0)`

**Complexidade:** ⭐

---

## Prioridade

| Feature | Complexidade | Valor | Prioridade |
|---------|-------------|-------|------------|
| Sensitivity | ⭐ | Médio | Alta |
| Button Long Press | ⭐⭐ | Alto | Média |
| Button Toggle | ⭐⭐ | Médio | Baixa |

---

## Nota

Estas features são **opcionais** e não bloqueiam o uso atual do firmware.
Implementar apenas quando houver demanda específica.
