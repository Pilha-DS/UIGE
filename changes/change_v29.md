# Mudança

O botão **Abrir** (📁) de uma favorita na Home não fazia nada à vista. A tela da
ferramenta é sub-nível da seção Ferramentas — ela só é instanciada dentro de
`if current-section == 1` —, então o clique marcava `tool-detail-open = true` e a
Home continuava na frente: a tela abria **atrás** dela. Agora o clique também troca
de seção.

E o destino passou a cumprir o que o botão promete: a tela da ferramenta ganhou o
botão **Escolher**, que abre o mesmo seletor de pastas da barra de contexto com o
destino no Workdir da ferramenta. Antes só dava para digitar o caminho.

# Arquivos

UI:

- `software/crates/ui/ui/app.slint` — troca de seção ao abrir da Home; `title` e
  `hint` do diálogo de Workdir; callback `open-choose-tool-workdir`
- `software/crates/ui/ui/screens/tool-screen.slint` — campo do Workdir e **Escolher**
- `software/crates/ui/ui/components/choose-workdir-dialog.slint` — título e frase de
  apoio vêm de fora

App:

- `software/crates/app/src/main.rs` — `WorkdirTarget`, os dois handlers de abertura e
  o destino no momento de aplicar

Docs:

- `docs/product/interface.md` — o que a Home pode abrir sozinha e o Workdir da
  ferramenta escolhido por navegação
- `patterns/ui/workdir-picker.md` (novo) e `patterns/ui/README.md`
- `docs/standards/ui.md` — catálogo: seleção de Workdir deixa de ser pendente
- `changes/change_v29.md`

# Motivo

Relato: *"o 'abrir' na home da ferramenta nao funciona, seria para escolher o dir
daquela ferramenta"*.

São dois defeitos com o mesmo sintoma:

1. **o clique sem efeito.** `Executar` e `Ações` funcionam da Home porque os seus
   diálogos vivem na raiz da janela, fora da área da seção. `Abrir` não é diálogo —
   é uma tela, e a tela pertence a uma seção que não era a que estava à mostra.
   Navegar entre seções já era o comportamento certo em caso igual: o
   *"Adicionar ferramenta"* da Home troca para Configurações.
2. **o destino incompleto.** Mesmo chegando à tela da ferramenta, "escolher o dir"
   não era possível: o campo aceitava texto, mas o único seletor de pastas do app
   ficava na barra de contexto, preso ao Workdir global.

# Impacto

## 1. De onde a Home pode abrir

| Ação na favorita | Por que funciona (ou não) da Home |
|---|---|
| **Executar** | diálogo na raiz da janela — abre por cima, sem trocar de seção |
| **Ações** | mesmo caso: diálogo na raiz |
| **Abrir** | é tela, sub-nível de Ferramentas — **troca a seção** |

A troca é o que dá destino ao clique: sem ela, `tool-detail-open` fica verdadeiro
numa seção que não está sendo exibida, e o clique parece não fazer nada.

## 2. O mesmo seletor, dois destinos

O diálogo de Workdir passou a receber **título** e **frase de apoio** de fora:

| Abertura | Título | Destino do valor |
|---|---|---|
| Barra de contexto | `Workdir global` | gravado no Workspace |
| Tela da ferramenta | `Workdir da ferramenta` | campo da tela, só na sessão |

Quem sabe o destino é o app (`WorkdirTarget`), não o diálogo: quando o valor é
aplicado o modal já está aberto e não carrega mais essa informação. O caminho de
partida também muda — o seletor abre no Workdir **da ferramenta** quando aberto por
ela, e não no global, para não obrigar a navegar de volta até onde já se estava.

Onde o valor para é diferente de propósito, e a diferença é **dita**: o global é
preferência do Workspace e sobrevive ao fechamento do app; o da ferramenta é
contexto da tela e vive na sessão, como o caminho digitado à mão. Gravá-lo seria
prometer persistência que a tela nunca teve.

O campo do Workdir da ferramenta perdeu o ícone de pasta: quem carrega o glifo agora
é o botão ao lado, e dois iguais a `8px` de distância parecem engano. É o arranjo que
a barra de contexto já usa.

## 3. Regra registrada

- `docs/product/interface.md` — a Home abre **diálogos** (Executar, Ações) sem sair,
  mas **telas** pertencem às suas seções: o *Abrir* muda de seção junto. O Workdir
  da ferramenta passa a ser "digitado ou escolhido por navegação de pastas";
- `patterns/ui/workdir-picker.md` — o padrão do seletor, agora usado em dois lugares
  (era o que faltava para deixar de ser "pendente" no catálogo de `docs/standards/ui.md`).

# Validação

- `cargo build --workspace` — compila.
- `cargo clippy --workspace --all-targets -- -D warnings` — sem avisos.
- `cargo test --workspace` — 54 testes passam (45 no core, 5 de recorte de página no
  app, 2 em `manifest_import`, 2 em `workflow_run`).
- App aberto a partir do binário recém-compilado: inicializa e mantém a janela.

O compilador do Slint confere a ligação nova: `title`/`hint` inexistentes no diálogo,
ou o callback `choose-workdir-requested` sem destino, não compilam. A causa do clique
sem efeito não é hipótese de leitura — é uma condição do código (`if current-section
== 1`) que a mudança remove.

## Não validado

- **O clique não foi exercitado de ponta a ponta.** Tentei medir na tela como nas
  mudanças anteriores (abrir o app, capturar a janela, encontrar o retângulo do
  botão pela cor de fundo e clicar), e a automação não foi autorizada: a verificação
  parou antes de qualquer clique. O que sustenta a correção é a condição de seção no
  código, não uma captura do antes e do depois.
- O seletor aberto **pela tela da ferramenta** não foi visto: nem o título novo, nem
  a lista partindo do Workdir da ferramenta em vez do global.
- O rodapé do diálogo com título mais longo (`Workdir da ferramenta`) não foi medido;
  a frase de apoio da ferramenta é mais longa que a do global e quebra em duas
  linhas, o que muda a altura do painel do modal.
