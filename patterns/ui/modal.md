# Modal

## Quando usar

- criar ou editar uma entidade em um fluxo curto (ex.: criar perfil);
- pedir confirmação ou dados adicionais sem sair da tela atual;
- isolar uma tarefa pontual da home / tela principal;
- operar um item específico de uma lista (ex.: a ferramenta do 📁), quando o
  contexto para voltar não muda.

## Quando não usar

- navegação principal entre áreas do app;
- formulários longos ou multi-etapa (preferir tela dedicada);
- quando o conteúdo do modal abre **outro** modal (ver *Um modal por vez*);
- feedback trivial que caiba em status inline.

## Estrutura / comportamento

1. **Backdrop** escurecido cobre a janela e bloqueia interação com o fundo.
2. **Painel** claro, do tamanho do conteúdo; texto com cores explícitas.
3. Montar o diálogo na home com `if open:`; no Modal usar `visible` (`@children` não pode ficar em `if`).
4. **Dismiss**: clique **apenas** no backdrop (fora do painel) ou em “Cancelar”.
5. Controles dentro do modal (campo/botões) devem ter estilo próprio legível em painel claro — não depender do tema escuro dos `std-widgets`.
6. Enquanto `busy`, o dismiss pelo backdrop fica desabilitado.
7. Ações primárias à direita (Cancelar · Salvar).
8. O painel precisa de um `TouchArea` absorvente atrás do conteúdo: texto/área vazia não captura ponteiro e o clique vaza para o backdrop.

### Painel limitado à janela

O painel cresce com o conteúdo **enquanto couber**, e para no teto quando não
couber:

```text
altura do painel = min(preferred-height do conteúdo, altura da janela - 2 * 24px)
largura do painel = min(panel-width, largura da janela - 2 * 24px)
```

Motivo: sem teto, um diálogo mais alto que a janela era desenhado a partir do
topo e o que passava da borda de baixo ficava **fora** — sem rolagem e sem como
alcançar o botão de confirmar. Em janela baixa (ou tela pequena) isso não é caso
raro.

O `margin` de `24px` mantém a folga até a borda: encostado, o painel deixa de
parecer flutuar.

### Corpo rolável, cabeçalho e rodapé fixos

O que rola é o **corpo**, não o painel inteiro: o título fica no topo e as ações
continuam no rodapé, sempre visíveis. Um modal em que é preciso rolar para achar
“Salvar” esconde justamente o que a pessoa procura.

Consequências na implementação:

- o corpo é um `ScrollView` com `preferred-height` vinda do **conteúdo**, e não do
  próprio `ScrollView` — ele declara `preferred-height: 100%` do pai, o que
  amarraria a altura do painel a ela mesma;
- `@children` cai dentro do `Flickable`, que **não é layout**: o corpo precisa de
  um `VerticalLayout` interno, senão um diálogo com vários irmãos (o seletor de
  Workdir tem) os empilha na mesma posição;
- esse layout interno declara `width: 100%`, que resolve contra o `Flickable`.

### Um modal por vez

O app mantém **no máximo um modal aberto**. Dois modais empilham dois fundos
escurecidos e dois botões de fechar, e não há como saber qual `Esc` fecha qual.

Duas formas de respeitar isso:

- **substituir a camada** — enquanto o seletor de pastas está aberto, o diálogo da
  ferramenta sai de cena e volta quando o seletor fecha (com o caminho aplicado);
- **promover a tela** — quando o conteúdo que abre modal é longo (montar uma
  sequência de etapas, por exemplo), o que era modal vira **tela** e a pilha passa
  a ser tela → modal. Padrão: [`modal-form.md`](modal-form.md).

## Variantes

- **Formulário curto** — campos + Cancelar/Salvar (ex.: criar perfil).
- **Painel de operação** — lista de ações + formulário de execução + histórico
  (ex.: diálogo da ferramenta).
- Futuro: confirmação destrutiva (padrão próprio quando necessário).

## Exemplo canônico

- Implementação: `software/crates/ui/ui/components/modal.slint`
- Uso: `software/crates/ui/ui/components/create-profile-dialog.slint`
- Uso com corpo longo: `software/crates/ui/ui/components/tool-dialog.slint`
