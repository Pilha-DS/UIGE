# ADR 0003 — Estrutura da interface

## Status

Aceito

## Contexto

A interface cresceu empilhando cada funcionalidade nova na mesma página, dentro de um único scroll. Hoje são nove blocos: Workspace, Workdir global, biblioteca de ferramentas, execução rápida, perfis, workflows, histórico, manifests e output. Isso produz três problemas:

1. **Modos de uso diferentes misturados** — operar, reutilizar e administrar disputam o mesmo espaço, sem indicar o que é uso diário e o que é área avançada.
2. **O fluxo principal está partido** — a ferramenta é escolhida num bloco e a ação é executada em outro, embora sejam um fluxo só.
3. **O resultado fica longe da ação** — o output está no fim de uma página longa.

O spec de referência já definia navegação por seções (`reference/current-specification.md`, seção 33: Home / Ferramentas / Perfis / Configurações), uma tela de ferramenta (39), uma execução rápida (38) e uma view de execução (60). Nada disso foi implementado: a UI cresceu sem seguir o modelo. O destino não estava indefinido, apenas não foi adotado.

Era preciso fixar a estrutura antes de continuar acrescentando telas.

## Decisão

Adotar navegação por seções, com barra de contexto persistente.

1. **Barra de contexto no topo** — seletor de Workspace e Workdir global, visíveis em todas as telas.
2. **Sidebar** com Home, Ferramentas, Perfis, Workflows e Configurações.
3. **Resultado no contexto** — status, exit code e output no painel de execução da tela que executou. Não existe tela global de Execuções.
4. **Histórico onde a execução nasce** — com a ferramenta ou com o workflow. A Home mostra apenas um resumo dos últimos.
5. **Barra de status na base** para mensagens curtas do aplicativo, separada do painel de execução.
6. **Curadoria de ferramentas em Configurações** — descoberta, importação, versões e edição de definição não ficam no caminho do uso diário.
7. **Profundidade máxima de três níveis** — seção → item → diálogo.

A definição da estrutura, tela por tela, está em [`../product/interface.md`](../product/interface.md).

### Desvio registrado: Workflows como seção própria

O modelo de domínio diz que **quem referencia Execution ou Workflow é o Profile** (`architecture/domain-model.md`), e `product/features.md` descreve um Perfil podendo representar uma execução única ou um workflow. Nesse modelo, o usuário veria apenas “Perfis”, e workflow seria um detalhe interno.

A decisão é manter **Perfis e Workflows como seções separadas**. O motivo é o estado atual: um Profile ainda configura somente uma Execution, e o vínculo `Profile → Workflow` não existe. Expor Workflows diretamente é o que reflete o que o sistema realmente sabe fazer hoje.

Este desvio é consciente e fica registrado. Se `Profile → Workflow` for implementado, a separação deve ser reavaliada — não porque a decisão estava errada, mas porque o motivo dela deixa de existir.

## Alternativas consideradas

- **Página única reorganizada.** Menor esforço e sem navegação nova, mas mantém os modos de uso misturados e o resultado longe da ação. Não resolve a causa.
- **Abas no topo.** Hierarquia equivalente à sidebar, porém pior com cinco seções e nomes de largura variável.
- **Tela global de Execuções.** Centralizaria todo o histórico, mas cria um segundo lugar para a mesma informação e reforça a distância entre ação e resultado.
- **Unificar Workflows sob Perfis (modelo canônico).** Exigiria implementar `Profile → Workflow` e retrabalhar a UI de Workflows recém-entregue. Adiado; o desvio acima registra a dívida.

## Consequências

- A hierarquia passa a existir: cada tela tem uma responsabilidade, e o contexto (Workspace, Workdir) fica sempre visível em vez de enterrado na página.
- **Custo:** a home atual (`software/crates/ui/ui/app.slint`) é praticamente substituída, e o Slint passa a precisar de um shell com troca de seções — hoje há só uma tela.
- Enquanto houver apenas o Workspace padrão, o seletor de Workspace fica sem função real. Deve ser implementado junto com os Workspaces de verdade, não antes.
- A área avançada (importação, versões, edição de definição) deixa de competir com o uso diário, mas fica a mais cliques de distância — trade-off aceito.
- Permanece em aberto: `Profile → Workflow`, modos de abertura de perfil e favoritos.
