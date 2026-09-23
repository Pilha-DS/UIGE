# Funcionalidades

Este documento descreve funcionalidades do produto. Termos técnicos e estruturas internas ficam em `../domain/` e `../architecture/`.

## Workspaces

O usuário pode criar ambientes de trabalho separados por contexto, projeto ou categoria.

Cada Workspace pode possuir:

- nome e ícone;
- Workdir global;
- ferramentas visíveis;
- perfis visíveis;
- favoritos;
- preferências próprias.

Deve existir um Workspace padrão. Uma mesma ferramenta ou perfil pode aparecer em vários Workspaces sem ser duplicado.

## Ferramentas

O usuário pode:

- visualizar ferramentas disponíveis;
- buscar ferramentas;
- abrir uma ferramenta;
- executar uma ação rapidamente;
- abrir a ferramenta em um Workdir específico;
- adicionar/remover de Workspaces;
- favoritar;
- criar um perfil a partir dela;
- abrir informações e configurações avançadas.

## Execução rápida

A execução rápida permite selecionar uma ação e seus parâmetros utilizando, por padrão, o Workdir global do Workspace ativo.

Ela representa uma única ação. Fluxos com múltiplas ações pertencem a Workflows/Perfis.

## Workdir

O produto deve distinguir:

- **Workdir global:** pertence ao Workspace ativo;
- **Workdir da ferramenta:** vale para a instância aberta daquela ferramenta;
- **Workdir do perfil:** pode ser fixo, solicitado ao abrir ou herdado do Workspace.

## Perfis

Um Perfil salva uma configuração reutilizável.

Ele pode representar:

- uma execução única;
- um workflow com várias etapas.

Modos de abertura:

- apenas abrir;
- pedir confirmação;
- executar automaticamente.

Um perfil pode aparecer em vários Workspaces sem duplicação.

## Workflows

Um Workflow executa etapas ordenadas. Inicialmente deve permitir ao menos:

- ordenar etapas;
- configurar parâmetros por etapa;
- definir Workdir quando necessário;
- parar ou continuar em caso de erro.

Capacidades futuras podem incluir retry, fallback, condições e compartilhamento de outputs entre etapas.

## Atalhos

O usuário pode criar atalhos externos para abrir/executar um Perfil, por exemplo por arquivo `.desktop` no Linux.

O atalho referencia um Perfil; ele não deve duplicar a configuração da execução.

## Manifests pela interface

O usuário avançado pode:

- criar uma ferramenta manualmente;
- detectar um executável;
- importar uma definição;
- editar a definição;
- usar modo simples ou avançado;
- consultar e restaurar versões anteriores.

## Histórico de definição

Alterações reais em uma definição de ferramenta geram uma nova versão. Restaurar uma versão antiga cria uma nova versão baseada nela; histórico anterior não é destruído.

## Automação — futuro

Uma automação deverá executar um Perfil de acordo com uma programação ou condição definida. Automação não deve apontar diretamente para uma definição técnica de ferramenta.
