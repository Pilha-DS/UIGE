# Mudança

Registra no `README.md` (raiz) que o projeto é uma **POC utilizável** e que,
portanto, **pode conter erros**.

Aviso colocado logo abaixo da assinatura do projeto, em bloco de citação, para não
passar despercebido, e detalhado na seção **Estado do projeto**: o que existe
funciona e é usável, mas o conjunto está incompleto e divergências devem ser
tratadas como defeito a relatar — não como comportamento garantido.

O aviso remete a `docs/product/interface.md` (o que a interface promete, incluindo
o que está pendente) e a `changes/` (o que já foi validado), para não transformar o
README em especificação.

O mesmo aviso, resumido, foi adicionado em `software/README.md`, que é por onde
quem vai rodar o código entra.

# Arquivos

- `README.md` — aviso de POC utilizável e seção **Estado do projeto**
- `software/README.md` — aviso resumido, apontando para a raiz

# Motivo

Quem encontra o repositório precisa saber, antes de usar, que está diante de uma
prova de conceito — o README descrevia o que o produto faz, mas não o grau de
maturidade.

# Impacto

- Nenhuma mudança de código, schema ou comportamento.
- A documentação canônica de produto e arquitetura continua em `docs/`; o README
  apenas resume e linka.
