# Princípios para IA

## 1. Respeitar a intenção existente

A IA deve melhorar organização e implementação sem reinterpretar silenciosamente o produto.

## 2. Menor mudança correta

Não refatorar áreas não relacionadas apenas porque parecem melhoráveis.

## 3. Fonte de verdade antes de suposição

Antes de criar um novo conceito, procurar se já existe definição em `docs/`, `patterns/` ou ADRs.

## 4. Explicar incompatibilidades

Se a solicitação conflitar com um padrão existente, a IA deve apontar o conflito e propor uma mudança explícita do padrão, em vez de ignorá-lo.

## 5. Segurança sem paralisia

Operações potencialmente perigosas devem ser sinalizadas e protegidas, mas isso não justifica adicionar complexidade desnecessária ao restante do sistema.

## 6. Não congelar o projeto

Padrões existem para dar consistência, não para impedir evolução. Mudanças estruturais válidas devem ser registradas por ADR e documentação correspondente.
