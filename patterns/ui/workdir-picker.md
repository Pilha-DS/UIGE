# Seleção de Workdir

## Quando usar

- quando a pessoa precisa apontar uma pasta que existe no disco, em vez de digitar o caminho de memória;
- em qualquer lugar que peça um diretório: o **Workdir global** do Workspace (barra de contexto) e o **Workdir de uma ferramenta** (diálogo da ferramenta).

## Quando não usar

- para escolher **arquivo**: o seletor lista só pastas, e um arquivo no meio da lista viraria uma linha que não leva a lugar nenhum;
- para um caminho que não é pasta de trabalho (URL, namespace, identificador interno): ali não existe "subir de nível", e a navegação por pastas não faz sentido;
- quando o valor é escolhido de uma lista fechada (Workspace, perfil): lista de opções e navegação de diretórios são coisas diferentes.

## Estrutura / comportamento

1. O **mesmo diálogo serve a todos os destinos**. Abrir dois diálogos quase iguais seria pior do que dizer a que Workdir ele serve.
2. Quem sabe o destino é o **app**, não o diálogo: o modal já está aberto quando o valor é aplicado, e nesse momento não carrega mais essa informação. Título e frase de apoio vêm de fora, junto com o caminho de partida.
3. **O campo é o ponto de partida e o valor final.** Ele mostra sempre o diretório atual, pode ser editado para ir direto a um caminho conhecido ("Ir") e é ele que o botão final aplica — quem digitou um caminho e confirmou não deveria precisar clicar em "Ir" antes.
4. A lista mostra as **subpastas** do diretório atual, e `..` na primeira posição quando existe pasta acima: subir e descer são a mesma ação, e a lista não precisa de um botão extra.
5. **Nada é selecionado** ao abrir: a primeira linha costuma ser `..`, e destacá-la sugeriria que subir é a escolha esperada. Percorrer as pastas não é escolher a última visitada — por isso o botão final aplica o campo, não a seleção.
6. Caminho inválido não fecha o diálogo: mostra o motivo ao lado do campo e mantém o que foi digitado, para correção.
7. Onde o valor vai parar muda com o destino, e isso é do app:

| Destino | O que acontece com o valor |
|---|---|
| Workdir global | preferência do Workspace: gravado, vale para as execuções simples |
| Workdir da ferramenta | contexto do diálogo: vive na sessão, como o caminho digitado à mão |

A diferença é dita no diálogo, não escondida: prometer persistência que a tela nunca teve seria pior do que não gravar.

8. **Um modal por vez.** Este diálogo é modal, e quem o abre também pode ser (o diálogo da ferramenta). Nesse caso a camada de baixo **sai de cena** enquanto o seletor está aberto e volta ao fechar, já com o caminho aplicado — em vez de ficar atrás dele com um segundo fundo escurecido ([`modal.md`](modal.md)).

## Variantes

Não há. O `..` já cobre a navegação para cima, e o campo editável cobre o salto para um caminho conhecido — duas saídas para o mesmo caso (voltar ao início, favoritos) seriam complicação sem ganho.

## Exemplo canônico

- Componente: `software/crates/ui/ui/components/choose-workdir-dialog.slint`
- Destino: `WorkdirTarget` em `software/crates/app/src/main.rs`
- Uso: barra de contexto e `software/crates/ui/ui/components/tool-dialog.slint`
