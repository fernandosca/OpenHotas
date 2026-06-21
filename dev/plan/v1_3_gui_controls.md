# OpenHOTAS — GUI Controls para V1.3 Features

> Implementação dos controles gráficos para axis-to-button e center offset.

---

## Status

| Feature | Firmware | CLI | GUI |
|---------|----------|-----|-----|
| Axis-to-Button | ✅ | ✅ | ❌ Pendente |
| Center Offset | ✅ | ✅ | ❌ Pendente |

---

## 1. Axis-to-Button — Controles GUI

### Arquivo: `gui/src/components/dashboard/Dashboard.tsx`

**Adicionar seção por eixo:**

```
┌─────────────────────────────────────────┐
│ Axis-to-Button                          │
├─────────────────────────────────────────┤
│ [Toggle] Habilitado                     │
│                                         │
│ Threshold                               │
│ [═══════════════════════] 80%           │
│                                         │
│ Direção                                 │
│ [Positive ▼]                            │
│                                         │
│ Button Index                            │
│ [28]                                    │
└─────────────────────────────────────────┘
```

**Componentes necessários:**
- `Switch` (Radix UI) — para habilitar/desabilitar
- `Slider` (Radix UI) — para threshold (0-100%)
- `Select` (Radix UI) — para direção (Positive/Negative/Both)
- `Input` (HTML) — para button index (0-31)

### Lógica

```typescript
// Update handler
const updateAxisToButton = (axisIndex: 0 | 1 | 2, partial: Partial<AxisToButtonConfig>) => {
  deviceConfig.updateAxis(axisIndex, {
    axis_to_button: {
      ...deviceConfig.config.axes[axisIndex].axis_to_button,
      ...partial,
    },
  });
};
```

### Validação

- `threshold`: 0-1000 (permille)
- `button_index`: 0-31
- `direction`: Positive | Negative | Both

---

## 2. Center Offset — Controles GUI

### Arquivo: `gui/src/components/dashboard/Dashboard.tsx`

**Adicionar seção por eixo:**

```
┌─────────────────────────────────────────┐
│ Center Offset                           │
├─────────────────────────────────────────┤
│ [-2.0%] [═════════|═══════════] [+2.0%]│
└─────────────────────────────────────────┘
```

**Componentes necessários:**
- `Slider` (Radix UI) — range -200 a +200 (permille)
- Display do valor em %

### Lógica

```typescript
// Update handler
const updateCenterOffset = (axisIndex: 0 | 1 | 2, value: number) => {
  deviceConfig.updateAxis(axisIndex, {
    center_offset_permille: value,
  });
};
```

### Validação

- `center_offset_permille`: -200..200
- Default: 0

---

## 3. Storybook Stories

### `gui/src/components/dashboard/Dashboard.stories.tsx`

Adicionar stories com axis-to-button e center offset configurados.

---

## Verificação

```sh
cd gui && npx tsc --noEmit && npm run build
```

---

## Nota

Esta implementação é **somente controles** — não altera a lógica de negócio.
A parte gráfica (layout, cores, posicionamento) fica a critério do designer.
