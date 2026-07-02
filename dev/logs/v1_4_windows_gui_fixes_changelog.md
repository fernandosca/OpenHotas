# OpenHOTAS V1.4 — Windows GUI Fixes Changelog

> Implementado em 2026-07-02 na branch `codex/windows-gui-fixes` após testes
> manuais da aplicação no Windows.

## Janela no Windows

- Removida a moldura nativa para evitar controles duplicados de minimizar e
  fechar junto à barra personalizada.
- A barra superior personalizada passou a permitir o arraste da janela.
- Liberadas explicitamente as permissões Tauri para fechar, minimizar e
  iniciar o arraste da janela.

## Conexão e configuração

- O estado da interface agora é atualizado imediatamente após conectar ou
  desconectar, sem depender de uma operação posterior de gravação.
- O polling de eixos e estatísticas é disparado após a conexão.
- A configuração ativa é recarregada ao conectar e após finalizar uma
  calibração.
- O salvamento de curvas deixa de sobrescrever uma calibração recente com a
  cópia local antiga dos limites.

## Calibração

- Erros de calibração agora descartam a sessão pendente no firmware usando a
  operação de finalização como cancelamento seguro.
- Após um `CalibrationError`, a interface retorna ao estado inicial e permite
  trocar o eixo ou reiniciar a calibração.
- A mensagem original do erro permanece visível para diagnóstico.

## Botões

- A grade de botões passou a exibir os índices `0..31`, alinhados ao protocolo,
  ao firmware e ao campo de configuração do botão virtual.

## Curvas

- Adicionado marcador em tempo real para indicar a posição atual do eixo no
  gráfico da curva selecionada.
- O marcador usa a saída processada pelo firmware e busca o ponto equivalente
  na curva, evitando aplicar a transformação uma segunda vez.
- Eixos configurados como invertidos são compensados na visualização.

## Validação

- Build TypeScript/Vite: OK.
- Build Storybook: OK.
- `cargo check` do backend Tauri: OK.
- Verificação de whitespace com `git diff --check`: OK.
- Build nativo anterior no Windows confirmou o toolchain Tauri/MSVC e gerou
  executável, instalador NSIS e MSI.
- Os artefatos locais de teste em `artifacts/windows-test` não fazem parte do
  versionamento.
