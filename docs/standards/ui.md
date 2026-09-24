# Padrões de UI/UX

## Princípio

A interface deve esconder complexidade incidental, não capacidade.

A direção visual e de interação inspira-se no GNOME HIG. Detalhes de stack: `docs/architecture/technology.md`.

## Modos

Quando necessário, preferir:

- modo simples como padrão;
- modo avançado para relações, riscos, permissões e edição técnica.

## Linguagem

Preferir termos orientados à tarefa. Ex.: mostrar “Nova ferramenta” em vez de `ManifestBuilder`.

## Como os padrões de componente nascem

Padrões concretos de componente **não são inventados de antemão**.

Eles são definidos **durante o desenvolvimento**, na primeira vez em que o componente for necessário de forma reutilizável.

Fluxo:

1. surge o uso real na UI;
2. extrai-se o padrão canônico;
3. documenta-se em `/patterns/ui/`;
4. usos seguintes reutilizam esse padrão.

Até existir o arquivo do padrão, não há implementação “oficial” daquele componente.

## Catálogo (a preencher)

Cada item abaixo deve ganhar um padrão em `/patterns/ui/` quando for usado de verdade. Enquanto estiver sem padrão, o status é **pendente**.

| Componente | Padrão | Status |
|---|---|---|
| Button | — | pendente |
| Modal | — | pendente |
| Form | — | pendente |
| Loading / erro / sucesso | — | pendente |
| Confirmação destrutiva | — | pendente |
| Output de execução | — | pendente |
| Seleção de Workdir | — | pendente |

Novos componentes recorrentes entram nesta tabela no mesmo momento em que o padrão for criado.

## O que o padrão deve registrar

Quando um padrão for definido, o arquivo em `/patterns/ui/` deve indicar ao menos:

- quando usar;
- quando não usar;
- estrutura / comportamento esperado;
- variantes permitidas (se houver);
- exemplo canônico.

Não duplicar a especificação completa do GNOME HIG aqui: linkar e adaptar ao UIGE.
