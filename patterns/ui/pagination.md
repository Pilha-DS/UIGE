# Paginação

## Quando usar

- em qualquer lista cujo tamanho **não é conhecido de antemão**: ferramentas, perfis, execuções de uma ferramenta, execuções de um workflow, candidatos de manifest descobertos;
- dentro de `ToolList` e `SelectList`, que já trazem o paginador embutido;
- quando a lista passa do tamanho de página (`10` itens) — abaixo disso o paginador não aparece.

## Quando não usar

- em lista de tamanho fixo e curto definido pela definição da ferramenta (ex.: as ações de uma ferramenta no painel de execução): paginar um conjunto de meia dúzia de itens só acrescenta um passo;
- em dropdown/seletor de contexto (ex.: seletor de Workspace na barra de contexto): um dropdown não pagina, e a lista é curta por natureza;
- para substituir o scroll de um painel de leitura (output de execução, etapas de workflow): ali o conteúdo é texto corrido, não itens escolhíveis.

## Estrutura / comportamento

1. O recorte é feito **no app** (Rust). A UI não consegue fatiar um modelo: ela recebe a página já pronta em `items` e o estado em `PageInfo`.

   ```text
   PageInfo {
       page        página exibida, base zero
       page-count  total de páginas
       total       itens na lista inteira (depois do filtro)
       offset      posição do primeiro item da página na lista inteira
   }
   ```

2. O índice que a lista informa de volta (`selected-index`, e os índices das ações de linha) é a posição na **lista inteira**. A lista soma `page-info.offset` para traduzir. Assim nenhum consumidor de seleção precisa saber que existe paginação.
3. A página é ajustada **antes** de exibir: se a lista encolheu (busca, filtro, remoção) vale a última página existente, não uma página vazia.
4. Busca ou filtro novos voltam para a primeira página. Uma lista que o app acabou de produzir (execução recém-gravada, workflow recém-criado) abre na página do item novo — criá-lo e não vê-lo seria o mesmo que não tê-lo criado.
5. O paginador só é instanciado quando há mais de uma página, e com `if`, não com `visible`: em Slint um item invisível **ainda ocupa lugar no layout**, e um paginador escondido deixaria um vão em toda lista curta.
6. Enquanto a paginação está à mostra, o botão desabilitado diz que não há para onde ir: `Anterior` na primeira página, `Próxima` na última.
7. O rótulo informa a faixa (`Página 2 de 5 · itens 11–20 de 42`), não só o número da página: a página sozinha não diz o tamanho do conjunto.

## Variantes

Não há. O tamanho de página é `10` para todas as listas — um valor por tela viraria dúvida sem ganho.

## Valores canônicos

| Propriedade | Valor |
|---|---|
| Itens por página | `10` |
| Altura do paginador | `30px` |
| Espaço entre lista e paginador | `6px` |

## Exemplo canônico

- Componente: `software/crates/ui/ui/components/pager.slint` (`PageInfo` + `Pager`)
- Recorte: `Pager` em `software/crates/app/src/main.rs`
- Uso: `software/crates/ui/ui/components/select-list.slint`, `software/crates/ui/ui/components/tool-list.slint`
