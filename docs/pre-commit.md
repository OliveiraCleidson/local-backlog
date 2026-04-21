# Pre-commit hook (opcional)

Este repositório não impõe hook local, mas `just pre-commit` roda o mesmo pipeline da CI (`fmt --check`, `clippy -D warnings`, `test`). Se quiser ativá-lo como hook de git, adicione o script abaixo e dê-lhe permissão de execução:

```sh
# .git/hooks/pre-commit
#!/bin/sh
set -e
just pre-commit
```

```sh
chmod +x .git/hooks/pre-commit
```

O hook é propositalmente simples — sem `pre-commit` framework, sem instalação extra. Se `just` não estiver no `PATH`, instale com `cargo install just` ou troque por `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test`.

## Bypass

Para pular a verificação em commits experimentais, use `git commit --no-verify`. Não use `--no-verify` em commits que vão para `main` — o mesmo pipeline roda na CI e vai falhar o PR.
