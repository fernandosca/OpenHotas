# OpenHOTAS V1.27 — GUI Changelog

> Implementado em 2026-06-20. Escopo limitado à interface gráfica do
> `hotas-configurator`.

## Objetivo

Registrar a reorganização visual feita após a primeira rodada da GUI V1.26,
com foco em reduzir excesso de informação, agrupar controles por contexto e
preparar a interface para uso como app desktop.

## Navegação

- A tela `Configuração` foi removida da navegação.
- Controles de eixo foram transferidos para `Eixos`.
- Configuração de botões foi transferida para `Botões`.
- A sidebar agora mantém somente as telas funcionais:
  - `Eixos`
  - `Botões`
  - `Curvas`
  - `Calibração`
  - `Diagnósticos`

Arquivos principais:

- `hotas-configurator/src/App.tsx`
- `hotas-configurator/src/components/dashboard/Dashboard.tsx`
- `hotas-configurator/src/components/buttons/ButtonsPage.tsx`

## Eixos

- Tela `Eixos` passou a concentrar configuração por eixo.
- Foram movidos para esta tela:
  - `Habilitado`
  - `Invertido`
  - `Reset EMA na deadzone`
  - `Filtros`
  - `Travel limits`
- Os controles foram consolidados em um único card por eixo.
- Foi preservado o bloqueio: eixo desabilitado não permite alterar parâmetros.
- Barra de alterações não salvas foi adicionada à tela para salvar/descartar
  direto do contexto de eixo.

Arquivo principal:

- `hotas-configurator/src/components/dashboard/Dashboard.tsx`

## Botões

- Card de configuração dos botões foi movido para a página `Botões`.
- A tela agora contém:
  - grid dos 32 botões
  - lista ativa
  - `Enabled mask`
  - `Inverted mask`
  - `Debounce`
- O espaçamento horizontal foi reduzido para manter os botões mais próximos e
  centralizados.

Arquivo principal:

- `hotas-configurator/src/components/buttons/ButtonsPage.tsx`

## Curvas

- O gráfico de curva deixou de ser editor de pontos.
- Pontos selecionáveis, arraste, clique, duplo-clique, presets por spline e
  histórico antigo foram removidos.
- O gráfico agora é somente visualização baseada em:
  - `expo_i16`
  - `deadzone_permille`
- Foram adicionados setups simples:
  - `Linear`
  - `Suave`
  - `Centro`
  - `S-curve`
- Slider de `Deadzone` foi mantido na tela.
- Botão `Desfazer` foi reintroduzido para desfazer mudanças momentâneas de
  setup/deadzone.
- Controles e gráfico foram reunidos em um único card.
- Labels foram posicionados acima dos grupos de função.
- O canvas foi ajustado para:
  - porcentagens mais nítidas usando `devicePixelRatio`
  - altura menor (`220px`)
  - comportamento somente leitura
- O bloco `Salvar`/`Descartar` fica abaixo do gráfico dentro do mesmo card e
  permanece visível mesmo sem alterações pendentes.

Arquivos principais:

- `hotas-configurator/src/components/curves/CurvePage.tsx`
- `hotas-configurator/src/components/curves/CurveEditor.tsx`

## Calibração

- Tela `Calibração` foi consolidada em um único card.
- Coluna lateral de status/notas foi removida.
- `Eixo` e `Status` ficam na mesma linha.
- O indicador textual do eixo atual foi removido do status.
- Badge de status ocupa a área da direita com altura alinhada ao seletor de
  eixo.
- Texto de instrução foi centralizado.
- Botão principal (`Iniciar`, `Capturar...`, `Finalizar`) foi alinhado à
  direita.
- Pontos `Mín`, `Centro` e `Máx` foram mantidos.
- Notas inferiores foram centralizadas, aumentadas e mantidas na mesma linha.

Arquivo principal:

- `hotas-configurator/src/components/calibration/Calibration.tsx`

## Diagnósticos

- Card `Saúde dos sensores` foi removido.
- Contagens de sensores foram transferidas para `Runtime stats`:
  - `Sensor X`
  - `Sensor Y`
  - `Sensor Twist`
- Tela foi consolidada em um único card.
- `Runtime stats` fica no topo.
- `Atualizar tudo` virou texto clicável abaixo de `Runtime stats`, alinhado à
  direita.
- `Ações do dispositivo` fica na parte inferior do mesmo card.
- Rótulos dos botões de ação foram centralizados.

Arquivo principal:

- `hotas-configurator/src/components/diagnostics/Diagnostics.tsx`

## Limpezas

- Removidos arquivos antigos da tela `Configuração`.
- Removida story da tela `Configuração`.
- Textos que apontavam para `Configuração` foram atualizados para `Eixos`.
- Removidas referências de edição visual por Puck.
- Storybook ficou fora do fluxo principal desta rodada; foco passou para
  inspeção direta no app/Chrome DevTools.

## Validação

Executado durante as mudanças:

```bash
cd hotas-configurator
npm run build
```

Resultado:

- Build da GUI: OK.
- Storybook não foi revalidado nesta rodada por decisão de focar no app em
  execução e no Chrome DevTools.

## Fora de Escopo

Este changelog não cobre:

- Firmware.
- `pc-cli`.
- Contrato binário do protocolo.
- Mudanças elétricas/hardware.
- Nova arquitetura visual completa.
