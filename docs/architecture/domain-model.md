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

## Profile

Um Profile configura uma execução ou workflow reutilizável e referencia as entidades necessárias. Ele não deve tornar o Manifest dependente do Profile.
