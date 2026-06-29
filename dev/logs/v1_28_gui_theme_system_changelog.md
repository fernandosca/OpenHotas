# OpenHOTAS V1.28 — GUI Theme System Changelog

> Implementado em 2026-06-29 na branch `codex/gui-theme-system`. Escopo
> limitado à interface gráfica em `gui`.

## Objetivo

Eliminar o acoplamento dos componentes a cores específicas e estabelecer um
contrato semântico único para temas. O visual existente foi preservado como
tema oficial `HUD`, permitindo adicionar novas identidades visuais sem alterar
a estrutura ou os ícones da interface.

## Infraestrutura de temas

- Criado `ThemeProvider` para controlar o tema ativo em runtime.
- A escolha é aplicada no atributo `data-theme` do elemento raiz.
- A preferência é persistida em `localStorage` com fallback seguro para `HUD`.
- Criado registro tipado dos temas disponíveis e do tema padrão.
- Adicionado seletor de aparência na tela `Configurações`.
- Ícones permanecem compartilhados e não fazem parte do contrato de temas.

Arquivos principais:

- `gui/src/theme/ThemeProvider.tsx`
- `gui/src/theme/themes.ts`
- `gui/src/components/settings/SettingsPage.tsx`
- `gui/src/main.tsx`

## Tokens semânticos

- Cores literais foram concentradas em `gui/src/theme/theme.css`.
- O contrato cobre:
  - fundos e níveis de superfície
  - bordas
  - texto principal, secundário, desabilitado e sobre accent
  - accent e estados `ok`, `warning` e `danger`
  - cores e trilhas dos eixos X, Y e Twist
  - cores específicas para canvas, gráficos e scrollbar
- O Tailwind passou a consumir variáveis do contrato, incluindo variantes com
  opacidade por canais RGB.
- Slots usados pelos componentes shadcn (`primary`, `secondary`, `muted`,
  `popover`, `card` e outros) também foram vinculados ao tema ativo.
- Classes `slate-*` foram removidas dos componentes e stories migrados.

Arquivos principais:

- `gui/src/theme/theme.css`
- `gui/tailwind.config.ts`
- `gui/src/index.css`
- `gui/src/theme/colors.ts`

## Temas oficiais

### HUD

- Formaliza o visual original do OpenHOTAS.
- Mantém superfícies azul-escuras, accent ciano e as cores atuais dos eixos.
- É o tema padrão para preservar o comportamento visual existente.

### Glass Cockpit

- Implementa a paleta definida no rascunho
  `dev/planos_rascunho/openhotas-design-system-v2.md`.
- Usa fundos oliva-escuros, textos dessaturados e accent azul-névoa.
- Atualiza cores de estados e eixos para tons de instrumentação civil com
  menor saturação.

## Canvas e gráficos

- Valores estáticos de `colors.ts` foram substituídos por resolução dos tokens
  ativos via `getComputedStyle`.
- HUD vetorial e editor de curvas redesenham quando o tema muda.
- Grid, referências, glow, marcadores, deadzone e cores dos eixos acompanham o
  tema selecionado.
- Recharts continua usando overrides CSS, agora alimentados pelo contrato de
  temas.

Arquivos principais:

- `gui/src/components/dashboard/Dashboard.tsx`
- `gui/src/components/curves/CurveEditor.tsx`
- `gui/src/theme/colors.ts`

## Storybook

- O preview global recebeu o mesmo `ThemeProvider` da aplicação.
- Stories foram migradas para tokens semânticos de texto.
- Componentes que dependem do tema podem ser renderizados isoladamente sem
  erro de contexto.

## Validação

Executado durante as mudanças:

```bash
cd gui
npm run build
npm run build-storybook
```

Resultado:

- Build TypeScript/Vite: OK.
- Build Storybook: OK.
- Verificação de whitespace com `git diff --check`: OK.
- Variantes Tailwind com opacidade confirmadas no CSS gerado.

## Fora de escopo

- Alteração ou substituição dos ícones.
- Firmware, CLI e protocolo binário.
- Mudanças estruturais de layout.
- Personalização de temas pelo usuário além das opções oficiais.
