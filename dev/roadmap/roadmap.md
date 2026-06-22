# OpenHOTAS — Roadmap

Este documento lista apenas funcionalidades ainda nao implementadas e que
continuam dentro do escopo atual do projeto.

---

## Proxima Fase

### 1. Button Toggle

Modo toggle para botoes fisicos.

```text
Modo normal: press = ligado, release = desligado
Modo toggle: press alterna entre ligado/desligado
```

Uso esperado:

- trem de pouso
- luzes
- flaps
- funcoes de simulador que mantem estado

Contrato previsto:

```rust
pub struct ButtonConfig {
    pub toggle_mask: u32,
}
```

Firmware:

- detectar borda de pressionamento
- alternar estado interno do botao
- aplicar toggle antes de publicar o report HID

GUI:

- adicionar controle por botao na tela Botoes
- manter o estado visual simples

Prioridade: alta

---

## Candidatos Depois do Teste de Hardware

### 2. Sensitivity Per Axis

Multiplicador de sensibilidade por eixo.

```text
80%  -> reduz ganho do eixo
100% -> comportamento atual
120% -> aumenta ganho com saturacao
```

Contrato previsto:

```rust
pub struct AxisConfig {
    pub sensitivity_permille: u16,
}
```

Faixa prevista:

```text
500..2000
```

Pipeline previsto:

```text
calibration -> center_offset -> sensitivity -> travel -> maxjump -> ema -> deadzone -> response
```

Prioridade: media

---

### 3. Button Long Press

Press longo para acionar uma funcao secundaria.

Antes de implementar, definir:

- se o destino sera um botao HID existente
- se o protocolo/HID passara a suportar mais de 32 botoes
- como evitar conflito com button toggle

Contrato em aberto:

```rust
pub struct ButtonConfig {
    pub long_press_threshold_ms: u16,
}
```

Prioridade: baixa

---

## Regras de Escopo

- manter foco em joystick
- manter 3 eixos: X, Y, Twist
- manter CDC como protocolo de configuracao
- manter `input_task` monolitica
- manter flash simples: uma configuracao ativa persistida
- implementar uma feature por ciclo de teste

---

## Fluxo Recomendado

```text
v1.3-test -> validar hardware
main      -> receber somente depois do merge
v1.4-test -> uma feature pequena por vez
```
