# Corpo de seção

## Quando usar

- envolver o interior de qualquer aba/seção que ocupe a área da seção em `app.slint` (Home, Ferramentas, Perfis, Workflows, Configurações, tela da ferramenta);
- sempre que o conteúdo for uma coluna de leitura: lista, formulário, painel de execução, texto corrido.

## Quando não usar

- dentro de um diálogo ou `Modal` — ali o painel já tem largura própria;
- como contêiner de barra (`Sidebar`, barra de contexto e barra de status), que ocupam a largura toda por definição;
- para limitar a largura de um bloco isolado (ex.: só o console): o corpo inteiro é a coluna, não uma parte dela.

## Estrutura / comportamento

1. O componente é um `Rectangle` transparente que preenche a área da seção (`width: 100%`, `height: 100%`).
2. Dentro dele, uma coluna com largura

   ```text
   min(parent.width - 2 * side-margin, max-content-width)
   ```

   ou seja: o recuo mínimo vale sempre; o teto só entra quando a janela é larga.
3. A coluna é centralizada por `x`, nunca alinhada à esquerda.
4. As `@children` entram **na coluna**, não no elemento raiz — é a coluna que limita a largura.
5. As telas não declaram recuo lateral próprio: o recuo é do corpo. Superfícies de grupo (`#f7f6f5`, `#f7f6f4`) vão até a borda da coluna, para delimitá-la.
6. O recuo vertical (topo e base) continua na tela, dentro da coluna.

## Valores canônicos

| Propriedade | Valor | Papel |
|---|---|---|
| `max-content-width` | `560px` | largura da coluna de leitura |
| `side-margin` | `24px` | recuo mínimo até a borda da área |

As telas não passam valor próprio: um corpo mais largo em uma aba e mais estreito em outra faria o conteúdo "pular" ao trocar de seção.

## Variantes

Não há. A coluna é única de propósito; se um caso real precisar de outra largura, ele deve virar discussão de padrão, não uma exceção local.

## Exemplo canônico

- Implementação: `software/crates/ui/ui/components/page-body.slint`
- Uso: `software/crates/ui/ui/screens/tools.slint`
