# OpenHOTAS Development Resources

Esta pasta concentra a documentacao de desenvolvimento do projeto.

## Estrutura

- `context/`: arquitetura, contratos, pinagem, hardware e regras de codigo.
- `docs/`: tutoriais pessoais locais. Esta pasta nao entra no Git.
- `logs/`: historico de builds, decisoes e mudancas por versao.
- `roadmap/`: planos revisados e versionados para implementacao.
- `planos_rascunho/`: planos que ainda dependem de revisao. Esta pasta nao entra no Git.

## Arquivos principais

- `logs/CHANGELOG.md`: o que mudou, por versão — formato Keep a Changelog, `[Unreleased]` sempre no topo.
- `logs/DECISIONS.md`: por que foi feito assim — decisões de design com motivo e status.
- `logs/RISKS.md`: riscos técnicos aceitos e sua condição de mitigação.
- `logs/old_log_arch/`: build logs brutos anteriores à consolidação acima. Fonte de auditoria histórica — não editar, não é lido pelo agente em fluxo normal.


## Regra simples

- Tutorial pessoal vai em `docs/` e fica local.
- Plano revisado vai em `roadmap/` e entra no Git.
- Plano que ainda depende de revisao vai em `planos_rascunho/` e fica local.
- O que ja aconteceu vai em `logs/CHANGELOG.md` (mudanca), `logs/DECISIONS.md` (decisao) ou `logs/RISKS.md` (risco aceito) — a cada sessao, direto em `[Unreleased]`.
- Build log bruto e anterior a esse formato vai em `logs/old_log_arch/`.
- O que define contrato/arquitetura vai em `context/`.