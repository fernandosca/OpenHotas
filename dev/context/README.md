# OpenHOTAS — dev/

Pasta de contratos arquiteturais e histórico de desenvolvimento.
Lida por IAs antes de qualquer intervenção no código.

---

## Ordem de Leitura Obrigatória

```
1. context/01_architecture.md        ← PRIMEIRO. Escopo, tasks, estrutura, naming.
2. context/02_hardware_specs.md      ← Protocolos SPI, MT6826S, MCP23S17.
3. context/03_hardware_pinout.md     ← Mapeamento de pinos. Ler antes de tocar em drivers.
4. context/04_software_contracts.md  ← Pipeline, flash, USB, regras absolutas.
5. context/05_coding_guidelines.md   ← Clean code para agentes. Ler antes de escrever código.
```

Após ler `context/`, consultar conforme necessidade:

```
roadmap/                 → Planos revisados prontos para implementação
planos_rascunho/         → Planos que ainda dependem de revisão
logs/                    → Por que as coisas são como são (histórico de decisões)
```

O arquivo `docs/dev_environment_setup.md` é guia **para o desenvolvedor** — não
para o agente. Cobre WSL2, Rust, probe-rs, usbipd e fluxo de desenvolvimento.

---

## Estrutura

```
dev/
├── README.md                        ← este arquivo (índice de dev/)
├── CLAUDE.md                        ← lido pelo agente automaticamente
├── docs/                            ← documentação do ambiente (local, fora do Git)
│   ├── dev_environment_setup.md     ← WSL2, Rust embedded, probe-rs
│   └── git_workflow.md             ← branching model + gate de validação
│
├── context/                         ← contratos estáveis
│   ├── 01_architecture.md           ← identidade, escopo, tasks, naming
│   ├── 02_hardware_specs.md         ← MT6826S, MCP23S17, protocolos SPI
│   ├── 03_hardware_pinout.md        ← mapeamento GPIO, barramentos
│   ├── 04_software_contracts.md     ← pipeline, flash, USB, regras proibidas
│   └── 05_coding_guidelines.md      ← clean code para agentes (Akita guidelines)
│
├── planos_rascunho/                 ← planos que ainda dependem de revisão (local, fora do Git)
│   ├── roadmap.md                   ← roadmap atual consolidado (em revisão)
│   ├── v2_roadmap.md                ← visão de V2
│   ├── axis_compensation.md         ← rascunho: compensação acoplada entre eixos
│   ├── macros.md                    ← rascunho: sequenciador de botões
│   └── github_release_auto_update.md ← rascunho: auto-update (adiado)
│
└── logs/                            ← registro imutável de versões entregues
    ├── v1_0_build.md                ← build inicial
    ├── v1_1_build.md                ← correções MT6826S + refatoração tasks
    ├── v1_2_build.md                ← safe Rust: Mutex SPI + deadzone sem raw pointer
    ├── …
    ├── v1_3_build.md                ← V1.3.0: Axis-to-Button, Center Offset, Burst Read
    └── v1_27_gui_changelog.md       ← changelog isolado da GUI
```

---

## Regras desta Pasta

### `context/` — Contratos Estáveis

- Reflete o estado **atual** do firmware
- Só muda com decisão explícita + registro no `logs/`
- Uma IA que escreve código usa estes arquivos como verdade

### `roadmap/` — Planos Revisados

- Um arquivo por feature
- Planos já revisados, prontos para implementação
- Quando implementado: mover para `logs/` com cabeçalho de encerramento
- Nunca deletar — mover

**Cabeçalho de encerramento ao mover para `logs/`:**
```markdown
> **IMPLEMENTADO** em V1.21 · Jun/2026 · Build limpo confirmado.
> Checklist completo: todos os itens marcados.
```

### `planos_rascunho/` — Planos em Revisão

- Ideias e planos que ainda dependem de revisão
- Fica local (fora do Git)
- Quando revisado e aprovado, mover para `roadmap/`

### `logs/` — Histórico Imutável

- Um arquivo por versão entregue
- Nunca editar após criado
- Contém: o que mudou, por que mudou, erros encontrados, decisões tomadas

---

## O que NÃO fazer

```
❌ Ler apenas um arquivo de context/ e ignorar os outros
❌ Ignorar a ordem de leitura
❌ Editar arquivos de logs/ (são imutáveis)
❌ Implementar feature sem plano revisado em roadmap/ primeiro
❌ Manter planos concluídos em roadmap/ (mover para logs/)
❌ Redefinir constantes localmente nos módulos do firmware
❌ Alterar context/ sem registrar a mudança em logs/
```

---

*OpenHOTAS · dev · Jun/2026*
