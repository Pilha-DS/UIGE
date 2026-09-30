# ADR 0002 — Abertura da interface por comando

## Status

Aceito

## Contexto

O UIGE foi definido como interface gráfica para ferramentas originalmente operadas por terminal. Durante a Fase 4 foi considerada a hipótese de uma **CLI headless**, operando o produto sem GUI. Essa hipótese foi descartada: o produto continua sendo exclusivamente de interface gráfica.

O que se quer é menor e distinto: poder **abrir a interface já no ponto desejado** a partir da linha de comando — a janela principal, ou a interface de uma ferramenta específica. O uso pretendido é conveniência e integração com atalhos externos, não execução sem GUI.

## Decisão

Manter o produto **apenas como interface gráfica** e usar um argumento posicional no binário para escolher o que a interface abre:

```text
uige           # abre a interface principal
uige <id>      # abre a interface já focada na ferramenta <id> (ex.: uige git)
```

Regras que acompanham a decisão:

1. **Não existe modo headless.** Todo uso passa pela GUI; nenhuma ação de ferramenta é executada sem interface.
2. **A interface é a mesma.** O argumento apenas decide o estado inicial (qual ferramenta está selecionada), não cria telas ou fluxos paralelos.
3. **A ferramenta é identificada pelo `id` do Manifest** (ex.: `git`), que é estável, em vez do nome exibido.
4. **Argumentos iniciados por `-` são reservados para flags.** O posicional é a ferramenta; flags ficam livres para uso futuro (`--help`, `--version`).
5. **`id` desconhecido não é erro fatal.** A interface principal abre e informa que a ferramenta não foi encontrada.
6. **Ambiente sem display não é caso suportado.** Deve falhar com mensagem clara, em vez de travar.
7. **Exit code refere-se apenas à abertura.** Como a execução acontece dentro da GUI, o resultado de uma ferramenta não é propagado como exit code do processo — isso não é um contrato do produto.
8. **Este é o ponto de entrada dos atalhos externos.** Um Launcher pode invocar o binário apontando para uma ferramenta, sem duplicar configuração.

Trade-off aceito da forma posicional: ela é a mais curta de digitar, mas o posicional tende a ficar ambíguo se novos alvos de abertura surgirem (workspace, perfil). Se isso acontecer, a evolução natural é um subcomando, o que exige revisão desta decisão.

## Alternativas consideradas

- **CLI headless (executar Profile/Workflow sem GUI):** descartada nesta decisão. Mudaria a natureza do produto, exigiria contrato de saída em stdout/stderr e obrigaria a mover a orquestração de casos de uso do `main.rs` para o core, sob pena de duplicar regra de domínio entre dois fronts.
- **Flag nomeada (`uige --tool <id>`):** mais explícita e menos ambígua que o posicional, mas mais verbosa para a única ação existente; foi preterida em favor da forma curta.
- **Subcomandos verbosos (`open`, `tool`) para uma ação só:** acrescenta cerimônia sem ganho, já que abrir é o comportamento padrão do binário.
- **Dois binários (GUI e terminal):** sem execução headless, o segundo binário não teria função própria.
- **Não aceitar argumentos:** manteria tudo como está, mas descartaria atalhos que abrem direto na ferramenta desejada.

## Consequências

- `vision.md` permanece fiel ao produto: uma interface gráfica. A conveniência de abertura por comando não a transforma em duas interfaces.
- **Não há segunda camada de apresentação**, então a orquestração de casos de uso pode continuar onde está. O pré-requisito de movê-la para o core, que a hipótese de CLI exigia, **deixa de existir**.
- A separação de fronteiras já valia antes e continua valendo: formatação e recorte de output (ex.: "últimas 5 linhas") ficam no front, não no core.
- Habilita o item 17 (launchers) a apontar para uma tela específica, reforçando Launcher como forma de acesso rápido.
- Restrição que permanece: sem servidor gráfico não há uso do produto.

## Histórico

- Versão anterior deste ADR adotava CLI headless em binário único. Revertida para esta decisão. O registro da mudança está em `changes/change_v18.md`.
