# Modelo de domínio técnico

## Manifest

Deve ser reutilizável e independente das preferências pessoais do usuário.

Pode definir:

- executável;
- comandos;
- parâmetros;
- flags;
- tipos;
- validações;
- relações;
- outputs;
- permissões;
- riscos.

Não deve armazenar:

- Workspace;
- Workdir pessoal;
- favoritos;
- Profiles;
- Launchers;
- Automations;
- histórico de execução do usuário.

## Workspace

O Workspace armazena referências. Não duplica Tool ou Manifest.

Estrutura conceitual:

```text
Workspace
├── id
├── name
├── icon
├── globalWorkdir
├── toolIds[]
├── profileIds[]
├── favorites[]
└── preferences
```

Regras:

- existe sempre um Workspace `Padrão`, criado na primeira abertura e que não pode ser removido;
- o nome é único sem diferenciar maiúsculas/minúsculas e acentos; a comparação é feita em Rust, não com `COLLATE NOCASE`, porque o SQLite só dobra ASCII (`Ação` e `AÇÃO` passariam como distintos);
- o Workspace **ativo** é uma preferência do app, guardada em `app_meta`. Se a chave apontar para um Workspace que não existe mais, o app cai no padrão em vez de falhar;
- `toolIds[]` é curadoria explícita: um Workspace novo começa com as ferramentas disponíveis, e a partir daí adicionar/remover é decisão do usuário. Remover uma ferramenta a retira também dos favoritos;
- `favorites[]` referencia Tools e é preferência do usuário, então vive no Workspace e **não** no Manifest.

**Status:** implementado, exceto `icon` e `preferences`.

## Profile

Um Profile configura uma execução ou workflow reutilizável e referencia as entidades necessárias. Ele não deve tornar o Manifest dependente do Profile.

## Workflow

Um Workflow é uma sequência ordenada de `WorkflowStep`. Ele referencia Tool e Command por etapa; não copia a definição do Manifest nem guarda preferências pessoais.

O Workflow **não** é referenciado diretamente pelo Workspace. Quem aponta para `Execution` ou `Workflow` é o Profile, e é o Workspace que referencia Profiles.

**Status:** o modelo acima é o alvo. Hoje um Profile configura apenas uma `Execution` e o vínculo `Profile → Workflow` ainda não existe — por isso a interface expõe Workflows como seção própria, desvio registrado em [`ADR 0003`](../decisions/0003-estrutura-da-interface.md).

Estrutura conceitual:

```text
Workflow
├── id
├── name
├── description
└── steps[]
    ├── id
    ├── toolId
    ├── commandId
    ├── parameters
    ├── workdir (opcional)
    └── onError (Stop | Continue)
```
