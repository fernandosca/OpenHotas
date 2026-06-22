# Axis Compensation (Compensação Automática de Eixos)

> Feature para futura implementação. Não é um plano de implementação detalhado.

---

## Visão Geral

Quando o usuário move o stick em um eixo, o firmware **compensa automaticamente** os outros eixos com ratios configuráveis. Acoplamento virtual entre eixos.

**Cenário principal:** Helicóptero — mover stick para direita (X+) compensa Twist automaticamente.

---

## Mecânica

```
Stick: X = +0.5 (direita)
Firmware calcula:
  Twist = X × ratio_twist = 0.5 × 0.4 = +0.20
  Y     = X × ratio_y     = 0.5 × 0.1 = +0.05

Output final:
  X = 0.50 (controle direto)
  Y = 0.05 (compensação)
  Twist = 0.20 (compensação)
```

---

## Matriz de Compensação

```
         De X    De Y    De Twist
Para X:  1.0     r_xy    r_xt
Para Y:  r_yx    1.0     r_yt
Para T:  r_tx    r_ty    1.0
```

**Defaults para helicóptero:**
- `r_tx = 0.4` (X influencia Twist em 40%)
- `r_ty = 0.2` (Y influencia Twist em 20%)
- Demais: `0.0`

---

## Parâmetros

| Parâmetro | Default | Range | Descrição |
|---|---|---|---|
| `enabled` | false | bool | Ativar compensação |
| `ratio_y_from_x` | 0.0 | -1.0 a 1.0 | X → Y |
| `ratio_t_from_x` | 0.4 | -1.0 a 1.0 | X → Twist |
| `ratio_x_from_y` | 0.0 | -1.0 a 1.0 | Y → X |
| `ratio_t_from_y` | 0.2 | -1.0 a 1.0 | Y → Twist |
| `ratio_x_from_t` | 0.0 | -1.0 a 1.0 | Twist → X |
| `ratio_y_from_t` | 0.0 | -1.0 a 1.0 | Twist → Y |
| `toggle_button` | 0 | 0–31 | Botão para ativar/desativar |

---

## Posição no Pipeline

```
Pipeline existente: cal → maxjump → ema → dz → expo → response
                                    ↓
                              AxisOutput (X, Y, Twist)
                                    ↓
                              ┌─────────────────┐
                              │ AxisCompensation │  ← NOVO: pós-filtro
                              └─────────────────┘
                                    ↓
                              GamepadReport → USB HID
```

Atua como **pós-filtro** — não perturba pipeline existente.

---

## Arquivos Envolvidos

| Arquivo | Ação | Descrição |
|---|---|---|
| `filters/axis_compensation.rs` | Criar | Filtro de compensação |
| `filters/mod.rs` | Modificar | Adicionar módulo |
| `constants.rs` | Modificar | Defaults de tuning |
| `tasks/input.rs` | Modificar | Chamar filtro |
| `config/settings.rs` | Modificar | Serializar parâmetros |

---

## Custo Estimado

| Recurso | Consumo |
|---|---|
| Flash código | ~150 bytes |
| RAM | ~28 bytes (6 floats + 1 bool) |
| CPU | ~2µs por ciclo |
| Linhas de código | ~65 |

---

## Casos de Uso

1. **Helicóptero:** Compensação de torque (X → Twist)
2. **Aviação:** Coordinated turn (Y → Twist)
3. **Personalizado:** Qualquer acoplamento entre eixos

---

*OpenHOTAS · Plan · Jun/2026*
