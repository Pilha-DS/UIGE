# Interface

Define a **estrutura** da interface: navegação, telas e o que cada uma concentra.

- Regras de componentes e interação (modal, formulário, cores, estados): [`../standards/ui.md`](../standards/ui.md) e [`/patterns/ui/`](../../patterns/ui/).
- Justificativa desta estrutura: [`../decisions/0003-estrutura-da-interface.md`](../decisions/0003-estrutura-da-interface.md).
- Termos exibidos ao usuário: [`../domain/glossary.md`](../domain/glossary.md).

## Princípios

1. **Uma tela, uma responsabilidade.** Iniciar (Home), operar (Ferramentas), reutilizar (Perfis), sequenciar (Workflows), administrar (Configurações).
2. **Contexto sempre visível.** Workspace e Workdir global ficam na barra de contexto, não no meio de uma tela.
3. **Resultado onde a ação acontece.** Status, exit code e output aparecem no painel de execução, junto do que foi executado.
4. **Histórico no contexto que o produziu.** Execuções de uma ferramenta ficam com ela; de um workflow, com ele. A Home mostra apenas um resumo dos últimos.
5. **Área avançada não ocupa o uso diário.** Edição de definição, versões e importação vivem em Configurações.
6. **Termos visuais, não internos.** Ver [`../domain/glossary.md`](../domain/glossary.md).
7. **Profundidade máxima de três níveis:** seção → item → diálogo. Um quarto nível indica que a tela precisa ser dividida.

## Estrutura

```text
┌───────────────────────────────────────────────────────────────────┐
│ [ Desenvolvimento ▼ ]   📁 ~/Projects/Omny                        │  barra de contexto
├───────────┬───────────────────────────────────────────────────────┤
│ ⌂ Home    │                                                       │
│ ⌘ Ferram. │                                                       │
│ ▶ Perfis  │              área da seção selecionada                │
│ ⇉ Workfl. │                                                       │
│ ⚙ Config. │                                                       │
├───────────┴───────────────────────────────────────────────────────┤
│ Pronto                                                            │  barra de status
└───────────────────────────────────────────────────────────────────┘
```

## Barra de contexto

Presente em todas as telas:

- **Seletor de Workspace** — lista os Workspaces e permite criar um novo. Trocar o Workspace atualiza ferramentas, perfis, workflows visíveis, favoritos e preferências. Renomear e remover ficam em Configurações.

- **Workdir global** — valor atual e ação de alterar. É o padrão de toda execução que não declare um Workdir próprio.

O Workdir global é editado **apenas** aqui: repetir o campo em Configurações criaria dois lugares para o mesmo valor.

## Telas

### Home

Ponto de partida, **não um depósito de funcionalidades**. Concentra acesso rápido:

- **Favoritos** — ferramentas marcadas, com as mesmas ações da lista de Ferramentas (executar, abrir, ações). As três abrem **diálogos**: nada aqui troca de seção, porque abrir não muda onde o usuário está.
- **Últimas execuções** — resumo do que rodou por último, para retomar o trabalho. Visão geral, não substitui o histórico de cada ferramenta ou workflow.
- **Criar** — nova ferramenta, novo perfil, novo workflow. A criação de Workflow é uma tela de Workflows: pedir daqui leva para lá.

Não contém: seleção de ferramenta, formulário de execução, importação de manifests.

### Ferramentas

Só a lista. A execução não mora aqui.

- Lista com busca e alternador **Do Workspace** / **Todas**.
- Cada item expõe três ações:

| Ação | O que faz |
|---|---|
| **Executar** | Abre o **diálogo de execução** e roda no Workdir global |
| **Abrir** | Abre o **diálogo da ferramenta**, com Workdir próprio |
| **Ações** | Menu da ferramenta: favoritar, adicionar/remover do Workspace, criar perfil |

Ações previstas mas ainda não implementadas (editar definição, versões, criar
atalho, informações, remover) ficam fora do menu em vez de aparecerem
desabilitadas: item inerte não informa.

#### Diálogo de execução

Execução simples: escolher a ação, informar o parâmetro e executar.

- Roda no **Workdir global**. Para outro diretório, o caminho é **Abrir**.
- O resultado aparece **no próprio diálogo**, que não fecha ao executar — dá para
  ajustar o parâmetro e rodar de novo sem reabrir.
- É o caminho do tiro único. Quando o trabalho é naquele diretório, a **tela da
  ferramenta** é o lugar certo: ali existe histórico.

#### Tela da ferramenta

Sub-nível de Ferramentas. Representa *ferramenta + Workdir da ferramenta*:

```text
┌───────────────────────────────────────────────────────────┐
│ [ ‹ Ferramentas ]   Git   ·   definição v3                 │
│ Workdir da ferramenta  [ ~/Projects/Omny        ] [Escolher]│
│                        herdado do global                   │
├───────────────────────────────────────────────────────────┤
│ Ações                                                     │
│ Clone · Status · Pull · Push · Commit · Branch · Checkout  │
├───────────────────────────────────────────────────────────┤
│ Ação        [ Pull ▼ ]                                     │
│ Parâmetros  ...                                            │
│                                       [ Executar ]         │
├───────────────────────────────────────────────────────────┤
│ Status: Success    Exit code: 0                            │
│ Already up to date.                                        │  painel de execução
├───────────────────────────────────────────────────────────┤
│ Execuções desta ferramenta                                 │
└───────────────────────────────────────────────────────────┘
```

O Workdir da ferramenta começa herdando o global e pode ser alterado ali, para
aquela ferramenta — digitando o caminho ou escolhendo por navegação de pastas. É o
**mesmo seletor** da barra de contexto com outro destino: o valor fica na
ferramenta e não toca o global. O destino é do app, não do diálogo: o título diz a
que Workdir ele serve, e o caminho escolhido aqui vive na sessão, sem ser gravado —
a tela nunca prometeu persistência para ele. Alterar aqui **não** muda o Workdir
global — o global é editado apenas na barra de contexto.

Carregar um perfil abre esta tela já com ferramenta, ação, parâmetro e Workdir
preenchidos. O Workdir do perfil é aplicado à ferramenta, não ao global.

### Perfis

Configuração reutilizável.

- Lista com busca; por item: executar, editar, adicionar ao Workspace, criar atalho, duplicar, excluir.
- Um perfil mostra ferramenta, ação, Workdir e modo de abertura.
- Criar perfil a partir do diálogo da ferramenta (já com o contexto) ou pelo diálogo da própria tela.

Hoje um perfil representa **uma execução única**. O vínculo `Perfil → Workflow` está em aberto — ver [`../decisions/0003-estrutura-da-interface.md`](../decisions/0003-estrutura-da-interface.md).

### Workflows

Sequência ordenada de etapas.

- Lista, criação e execução.
- **Criação é uma tela**, não um diálogo: a criação se compõe de outra coisa que já é modal — detalhar uma etapa. Tela + modal é uma pilha legível; modal sobre modal não é.
- Cada etapa é **detalhada em modal** ao ser adicionada: ferramenta, ação, parâmetro, Workdir da etapa e o que fazer se falhar. A etapa não nasce de valores fixos na tela.
- Detalhe mostra a **definição** das etapas e o **resultado da execução** de cada uma, com o output no painel de execução.
- Execuções anteriores ficam nesta tela.

### Configurações

Área administrativa, separada do uso diário:

- **Workspace** — nome (renomear) e remoção. O Workspace padrão não pode ser removido. Ferramentas e perfis do Workspace são geridos na tela de Ferramentas; o Workdir global, na barra de contexto.
- **Ferramentas (avançado)** — pasta de descoberta, descobrir, importar, versões da definição, editar definição.

## Barra de status

Faixa na base da janela para mensagens curtas do aplicativo (“Perfil salvo”, “3 arquivos encontrados”, falha de validação). É onde vive o `status` atual da página única.

Não é onde o resultado de execução aparece: output, exit code e estado da execução pertencem ao painel de execução, dentro da tela que executou.

## Mapa: de onde sai cada coisa

A página única atual concentra nove blocos. Este é o destino de cada um:

| Hoje (página única) | Destino |
|---|---|
| `Workspace: Padrão` (rótulo fixo) | barra de contexto — seletor de Workspace |
| Workdir global (campo + Salvar) | barra de contexto |
| Biblioteca de ferramentas (combo) | Ferramentas — lista com busca |
| Execução rápida (ação + parâmetros + Executar) | Ferramentas — diálogo de execução; diálogo da ferramenta — painel de execução |
| Status · Exit code · Output | diálogo da ferramenta — painel de execução (status do app vai para a barra de status) |
| Perfis (combo + criar) | Perfis |
| Workflows (combo + criar + executar + etapas + execuções) | Workflows |
| Histórico (execuções) | no contexto: diálogo da ferramenta e Workflows; resumo na Home |
| Manifests (pasta, descobrir, importar) | Configurações — Ferramentas (avançado) |

## Estado da implementação

| Elemento | Estado |
|---|---|
| Shell (barra de contexto + sidebar + área da seção + barra de status) | implementado |
| Barra de status | implementado |
| Seletor de Workspace | implementado (selecionar, criar, renomear, remover) |
| Workdir global na barra de contexto | implementado |
| Home | implementado (execuções recentes + atalhos de criação) |
| Ferramentas | implementado (lista com busca, filtro Do Workspace / Todas, favoritos e ações por linha: Executar, Abrir, Ações) |
| Diálogo de execução | implementado (acionado por Executar; roda no Workdir global; resultado no próprio diálogo) |
| Diálogo da ferramenta | implementado (acionado por Abrir, na lista ou na Home; Workdir próprio herdado do global, digitado ou escolhido por navegação de pastas + painel de execução + histórico da ferramenta) |
| Menu de ações da ferramenta | parcial — só favoritar, adicionar/remover do Workspace e criar perfil |
| Painel de execução | implementado |
| Perfis | implementado (lista + criar; modos de abertura e atalho pendentes) |
| Workflows | implementado (lista + criação em tela, com etapa detalhada em modal + executar + etapas + execuções) |
| Configurações | parcial — Workspace (nome e remoção) e curadoria de definições; preferências pendentes |

Edição de definição, versões, criar atalho, informações e remover ferramenta continuam pendentes.
