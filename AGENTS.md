# AGENTS.md — Regras operacionais do UIGE

Estas regras se aplicam a qualquer IA/agente que modifique este repositório.

## Prioridade

Em caso de conflito, seguir nesta ordem:

1. pedido explícito do mantenedor;
2. decisões registradas em `docs/decisions/`;
3. padrões em `docs/standards/`;
4. documentação de arquitetura;
5. padrões concretos em `/patterns/`;
6. implementação existente.

Não ignorar um conflito silenciosamente.

## Princípios obrigatórios

- Fazer a menor mudança suficiente para atender ao objetivo.
- Não remover funcionalidades existentes sem solicitação explícita.
- Não alterar áreas não relacionadas apenas para “limpar” o código.
- Preferir soluções simples, legíveis e previsíveis.
- Reutilizar padrões existentes antes de criar um novo.
- Manter separação entre produto, domínio, arquitetura e UI.
- Preservar compatibilidade de dados/schema quando razoável.
- Tratar segurança e ações destrutivas explicitamente.
- Não colocar funcionalidade de uma arquitetura em outra, tipo, front no motor, ou regra no front



## Antes de implementar

A IA deve localizar e ler os documentos relevantes. Não assumir que o README contém toda a especificação.

Para uma mudança de:

- funcionalidade → ler `docs/product/`;
- conceito → ler `docs/domain/`;
- estrutura → ler `docs/architecture/`;
- convenção → ler `docs/standards/`;
- decisão central → consultar `docs/decisions/`.



## Arquitetura

Não criar novas camadas, padrões ou serviços apenas por preferência arquitetural.

Não inverter relações canônicas sem uma decisão explícita. Exemplos:

- Workspace referencia Tool/Profile; não os duplica.
- Profile referencia a definição/ações necessárias; Manifest não armazena Profiles.
- Launcher referencia Profile; não replica Commands/Parameters/Workflow.
- Automation referencia Profile; não aponta diretamente para Manifest.



## Documentação

Toda regra nova deve ser colocada no documento canônico correto.

Não duplicar a mesma especificação em vários arquivos. Quando necessário, adicionar link.

Mudanças relevantes devem gerar um novo arquivo em `/changes/`. Nunca sobrescrever ou apagar um change anterior para esconder histórico.

## Implementação

- Não adicionar dependência sem necessidade.
- Não alterar API/schema público silenciosamente.
- Não expor detalhes técnicos na UI sem necessidade.
- Não construir comando de shell por concatenação insegura quando argumentos estruturados puderem ser usados.
- Não guardar segredo em código, manifest, fixture ou log.



## Validação

Após uma mudança, executar checks aplicáveis ao escopo: testes, lint, build, schema validation ou validação manual reproduzível.

Se algo não puder ser validado, declarar explicitamente o que não foi validado.

## Entrega

Ao finalizar, informar de forma objetiva e direta:

- o que mudou;
- arquivos principais;
- decisões relevantes;
- validações executadas;
- riscos ou pontos pendentes.

