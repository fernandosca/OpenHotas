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
plan/                    → O que deve ser implementado agora
logs/                    → Por que as coisas são como são (histórico de decisões)
```

O arquivo `docs/dev_environment_setup.md` é guia **para o desenvolvedor** — não
para o agente. Cobre WSL2, Rust, probe-rs, usbipd e fluxo de desenvolvimento.

---

## Estrutura

```
dev/
├── README.md                        ← este arquivo
├── CLAUDE.md                        ← lido pelo agente automaticamente
├── docs/                            ← documentação do ambiente
│   └── dev_environment_setup.md     ← WSL2, Rust embedded, probe-rs
│
├── context/                         ← contratos estáveis
│   ├── 01_architecture.md           ← identidade, escopo, tasks, naming
│   ├── 02_hardware_specs.md         ← MT6826S, MCP23S17, protocolos SPI
│   ├── 03_hardware_pinout.md        ← mapeamento GPIO, barramentos
│   ├── 04_software_contracts.md     ← pipeline, flash, USB, regras proibidas
│   └── 05_coding_guidelines.md      ← clean code para agentes (Akita guidelines)
│
├── plan/                            ← features pendentes de implementação
│   ├── 1_21 cdc serial.md           ← CDC Serial via USB
│   ├── 1_21 usb flash seguro.md     ← USB flashing seguro
│   ├── v1_21_versioning.md          ← versionamento SemVer + BCD USB
│   └── v2_roadmap.md                ← visão de futuro (não é plano detalhado)
│
└── logs/                            ← registro imutável de versões entregues
    ├── v1_0_build.md                ← build inicial
    ├── v1_1_build.md                ← correções MT6826S + refatoração tasks
    ├── v1_2_build.md                ← safe Rust: Mutex SPI + deadzone sem raw pointer
    └── v1_21_build.md               ← V1.21 CDC Serial + flash seguro
```

---

## Regras desta Pasta

### `context/` — Contratos Estáveis

- Reflete o estado **atual** do firmware
- Só muda com decisão explícita + registro no `log/`
- Uma IA que escreve código usa estes arquivos como verdade

### `plan/` — Features Pendentes

- Um arquivo por feature
- Quando implementado: mover para `log/` com cabeçalho de encerramento
- Nunca deletar — mover

**Cabeçalho de encerramento ao mover para `log/`:**
```markdown
> **IMPLEMENTADO** em V1.21 · Jun/2026 · Build limpo confirmado.
> Checklist completo: todos os itens marcados.
```

### `log/` — Histórico Imutável

- Um arquivo por versão entregue
- Nunca editar após criado
- Contém: o que mudou, por que mudou, erros encontrados, decisões tomadas

---

## O que NÃO fazer

```
❌ Ler apenas um arquivo de context/ e ignorar os outros
❌ Ignorar a ordem de leitura
❌ Editar arquivos de log/ (são imutáveis)
❌ Criar features sem arquivo em plan/ primeiro
❌ Manter planos concluídos em plan/ (mover para log/)
❌ Redefinir constantes localmente nos módulos do firmware
❌ Alterar context/ sem registrar a mudança em log/
```

---

*OpenHOTAS · dev · Jun/2026*
