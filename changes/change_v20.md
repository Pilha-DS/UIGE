# Mudança

Definição básica da interface: estrutura de navegação e telas, documentada antes de implementar. Nenhum código foi alterado.

# Arquivos

- `docs/product/interface.md` (novo) — estrutura canônica da interface
- `docs/decisions/0003-estrutura-da-interface.md` (novo)
- `docs/decisions/README.md`
- `docs/product/README.md`
- `docs/product/flows.md`
- `docs/standards/ui.md`
- `docs/architecture/domain-model.md`
- `changes/change_v20.md`

# Motivo

A UI cresceu empilhando cada funcionalidade nova na mesma página. O `app.slint` chegou a nove blocos num único scroll: Workspace, Workdir global, biblioteca de ferramentas, execução rápida, perfis, workflows, histórico, manifests e output.

Três problemas decorrem disso:

1. modos de uso diferentes (operar, reutilizar, administrar) misturados, sem separar uso diário de área avançada;
2. o fluxo principal partido em dois — a ferramenta é escolhida em um bloco e a ação é executada em outro;
3. o resultado longe da ação — o output está no fim de uma página longa.

A causa raiz não é falta de definição: o spec de referência já descrevia navegação por seções (seção 33), tela de ferramenta (39), execução rápida (38) e view de execução (60). O modelo existe e não foi seguido.

# Impacto

## Decisão (ADR 0003)

- barra de contexto persistente no topo: seletor de Workspace + Workdir global;
- sidebar: Home, Ferramentas, Perfis, Workflows, Configurações;
- resultado da execução no contexto, no painel de execução da tela que executou — **não** existe tela global de Execuções;
- histórico onde a execução nasce; a Home mostra só um resumo dos últimos;
- barra de status na base para mensagens do aplicativo, separada do painel de execução;
- curadoria de ferramentas (descobrir, importar, versões, editar definição) em Configurações;
- profundidade máxima de três níveis.

## Desvio registrado: Workflows como seção própria

`domain-model.md` diz que **quem referencia `Execution` ou `Workflow` é o Profile**, e `features.md` descreve um Perfil podendo representar um workflow. Nesse modelo o usuário veria apenas “Perfis”.

A decisão é manter **Perfis e Workflows separados**, porque hoje um Profile configura apenas uma Execution e o vínculo `Profile → Workflow` não existe. O desvio foi registrado explicitamente no ADR e anotado em `domain-model.md`, conforme a exigência de não inverter relação canônica sem decisão explícita e de não deixar conflito silencioso.

O motivo do desvio é temporário por natureza: se `Profile → Workflow` for implementado, a separação deve ser reavaliada.

## Documentos alterados

- `product/interface.md` é novo e passa a ser a fonte canônica da **estrutura** da interface. Inclui um mapa de migração dizendo para onde vai cada um dos nove blocos da página única, e uma tabela de estado (implementado / parcial / pendente) para deixar claro que quase tudo ainda não existe.
- `standards/ui.md` continua tratando de componentes e interação, e agora linka `product/interface.md` para estrutura, evitando duplicação.
- `product/flows.md` foi alinhado às telas. O Workspace deixou de ser uma etapa dos fluxos: virou contexto, coerente com a barra de contexto.
- `architecture/domain-model.md` recebeu a nota de status sobre `Profile → Workflow`.
- Índices `product/README.md` e `decisions/README.md` atualizados.

## Consequências

- Custo alto na implementação: a home atual é praticamente substituída e o Slint passa a precisar de um shell com troca de seções, que hoje não existe.
- O seletor de Workspace fica sem função real enquanto houver apenas o Workspace padrão; deve ser implementado junto com Workspaces reais.
- A área avançada deixa de competir com o uso diário, mas fica a mais cliques de distância.

# Validação

Somente documentação foi alterada — nenhum arquivo em `software/` foi tocado, portanto não havia build nem teste a executar. Links entre os documentos novos e alterados foram conferidos manualmente.

**Não validado:** a estrutura proposta não foi implementada, logo não há verificação visual. A página única atual permanece no ar até que a implementação aconteça.
