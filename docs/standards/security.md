# Segurança

## Comandos

Toda execução deve tratar argumentos como dados estruturados sempre que possível.

## Risco

Manifests podem classificar ações/parâmetros por risco. A UI deve usar essa informação para solicitar confirmação ou permissão quando necessário.

## Privilégios

Elevação de privilégio deve ser explícita. O UIGE não deve esconder do usuário que uma ação exige permissões especiais.

## Segredos

Segredos não devem ser:

- gravados em logs;
- commitados;
- armazenados em manifests compartilháveis;
- exibidos em output sem necessidade.

## Ações destrutivas

Devem possuir confirmação apropriada e contexto suficiente para o usuário entender o impacto.
