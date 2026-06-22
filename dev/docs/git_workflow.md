# OpenHOTAS Git Workflow

Este guia resume o fluxo recomendado para trabalhar sem quebrar a branch principal.

## Ideia principal

- `main` e a versao estavel.
- `vX.Y-test` e o ambiente de teste da proxima versao.
- Pull Request e a ponte entre teste e `main`.
- Release nasce de uma tag criada na `main`.

Nunca trabalhe direto na `main` para features grandes.

## Comecar uma nova versao de teste

Sempre comece atualizando a `main`:

```bash
git switch main
git pull origin main
```

Crie uma branch de teste:

```bash
git switch -c v1.4-test
```

Suba a branch para o GitHub:

```bash
git push -u origin v1.4-test
```

No GitHub, abra um Pull Request:

```text
v1.4-test -> main
```

## Trabalhar na branch de teste

Veja a branch atual:

```bash
git branch --show-current
```

Veja o que mudou:

```bash
git status
```

Adicione arquivos ao commit:

```bash
git add caminho/do/arquivo
```

Ou, se revisou tudo antes:

```bash
git add .
```

Crie o commit:

```bash
git commit -m "feat: descricao curta da mudanca"
```

Envie para o GitHub:

```bash
git push
```

## Validar antes do merge

Antes de fazer merge para `main`, confirme:

- CI verde no Pull Request.
- Firmware compila.
- CLI compila.
- GUI compila.
- Teste de hardware passou, quando aplicavel.

Comandos uteis:

```bash
cargo clippy -p openhotas-protocol -- -D warnings
cargo clippy -p openhotas-cli -- -D warnings
cargo clippy --manifest-path firmware/Cargo.toml --target thumbv8m.main-none-eabihf -- -D warnings
cd gui && npm run build
```

## Fazer merge para main

Quando estiver tudo certo:

1. Abra o Pull Request no GitHub.
2. Confirme que os checks passaram.
3. Use `Squash and merge`.

Depois atualize sua `main` local:

```bash
git switch main
git pull origin main
```

## Criar release

Crie tag apenas depois do merge na `main`.

Exemplo:

```bash
git switch main
git pull origin main
git tag v1.4.0
git push origin v1.4.0
```

Tags que comecam com `v` disparam o workflow de release.

## Apagar branch de teste depois do merge

Depois que a versao entrou na `main`, a branch de teste pode ser apagada.

No GitHub, use `Delete branch` no Pull Request fechado.

Localmente:

```bash
git branch -d v1.4-test
git remote prune origin
```

## Recuperar a branch de teste atual

Para voltar ao ambiente de teste:

```bash
git switch v1.3-test
```

Para voltar a versao estavel:

```bash
git switch main
```

## Regra simples

```text
main       = estavel
vX.Y-test  = laboratorio
PR         = revisao + CI
tag vX.Y.Z = release
```
