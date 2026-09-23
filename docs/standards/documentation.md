# Padrão de documentação

## Fonte única de verdade

Uma regra deve ter um documento canônico. Outros documentos devem referenciá-la por link em vez de copiá-la.

## Separação obrigatória

- funcionalidade → `docs/product/`;
- termo/conceito → `docs/domain/`;
- implementação/estrutura → `docs/architecture/`;
- regra transversal → `docs/standards/`;
- decisão e trade-off → `docs/decisions/`;
- exemplos → `/patterns/`.

## README raiz

O README raiz deve permanecer curto e responder:

1. o que é o UIGE;
2. qual problema resolve;
3. estado do projeto;
4. princípios essenciais;
5. onde encontrar documentação;
6. como contribuir.

Ele não deve conter a especificação inteira do sistema.

## Atualização

Mudanças de comportamento devem atualizar a documentação correspondente no mesmo conjunto de alterações.
