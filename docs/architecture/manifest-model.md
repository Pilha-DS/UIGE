# Modelo de Manifest

Este documento define responsabilidades do formato. O schema exato deve possuir versionamento próprio.

## Parameter

Campos básicos esperados:

- `id`;
- `name`;
- `type`;
- `flag` quando aplicável;
- `required`;
- `default`;
- restrições de valor.

Relações avançadas possíveis:

- `requires`;
- `conflicts`;
- `oneOf`;
- `requiresOneOf`;
- `when`;
- `source`;
- `separator`;
- `output`;
- `risk`;
- `permissions`.

## Versionamento

- primeira versão: `v1`;
- nova versão somente quando o conteúdo realmente muda;
- restaurar uma versão anterior gera uma nova versão;
- histórico existente nunca é sobrescrito silenciosamente.

Exemplos canônicos devem ficar em `/patterns/manifests/`.
