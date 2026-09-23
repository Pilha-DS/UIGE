# UIGE — Especificação Geral do Sistema

## 1. Visão geral

O **UIGE** é uma interface gráfica para ferramentas originalmente utilizadas por terminal.

Seu objetivo é permitir que o usuário utilize ferramentas como:

```text
Git
Docker
Nmap
FFmpeg
Curl
Pacman
Systemctl
Maven
Kubectl
```

através de interfaces geradas a partir de definições estruturadas.

O UIGE deve ser:

```text
simples para executar
organizado para trabalhar
poderoso para configurar
extensível para automatizar
```

A regra central da interface é:

> **O usuário vê ferramentas e ações. O UIGE cuida de manifests, parâmetros, contextos, relações e execução por baixo.**

---

# 2. Arquitetura conceitual

Os principais conceitos são:

```text
Manifest
Tool
Workspace
Command
Parameter
Execution
Workflow
Profile
Launcher
Automation
```

Cada conceito possui uma responsabilidade específica.

```text
Manifest
=
define como uma ferramenta funciona


Tool
=
representa uma ferramenta disponível no UIGE


Workspace
=
organiza ferramentas, Profiles, favoritos e Workdir


Command
=
representa uma operação da Tool


Parameter
=
representa uma entrada de um Command


Execution
=
execução de um único Command


Workflow
=
sequência de Executions


Profile
=
configuração reutilizável de uma Execution ou Workflow


Launcher
=
forma rápida de abrir um Profile


Automation
=
forma futura de executar um Profile automaticamente
```

---

# 3. Manifest

O **Manifest** é a definição técnica de uma ferramenta.

Exemplo:

```text
Git
 ↓
git.manifest
```

Ele pode definir:

```text
executável
commands
parameters
flags
types
validations
relations
outputs
permissions
risks
```

O Manifest deve ser reutilizável e independente das preferências do usuário.

Por isso ele NÃO deve armazenar:

```text
Workspace
Workdir pessoal
favoritos
Profiles
Launchers
Automation
histórico de execução
```

---

# 4. Tool

Uma **Tool** representa uma ferramenta registrada e disponível no UIGE.

Exemplos:

```text
Git
Docker
Nmap
FFmpeg
Curl
Pacman
Systemctl
```

Nome visual:

```text
Ferramenta
```

Nome interno:

```text
Tool
```

Uma Tool utiliza um Manifest.

```text
Manifest
   ↓
 Tool
```

Exemplo:

```text
git.manifest
     ↓
    Git
```

---

# 5. Command

Um **Command** representa uma operação disponível dentro de uma Tool.

Exemplo:

```text
Tool:
Git

Commands:

Clone
Pull
Push
Commit
Status
Branch
Checkout
```

Internamente:

```text
Command
```

---

# 6. Parameter

Um **Parameter** representa uma entrada utilizada por um Command.

Exemplo:

```text
git clone
```

pode possuir:

```text
repository
directory
depth
recurse-submodules
```

Um Parameter pode possuir:

```text
id
name
type
flag
required
default
min
max
allowedValues
```

E regras avançadas:

```text
requires
conflicts
oneOf
requiresOneOf
when
source
separator
output
risk
permissions
```

---

# 7. Workspace

O **Workspace** representa um ambiente organizado dentro do UIGE.

Ele permite separar ferramentas e configurações por contexto.

Exemplo:

```text
Desenvolvimento

Git
Docker
Maven
Curl
```

Outro:

```text
Segurança

Nmap
Gobuster
Nikto
Whois
Curl
```

Outro:

```text
Mídia

FFmpeg
ImageMagick
yt-dlp
```

---

# 8. Responsabilidade do Workspace

Um Workspace pode definir:

```text
nome
ícone
Global Workdir
Tools visíveis
Profiles visíveis
Favoritos
Preferências
```

Estrutura conceitual:

```text
Workspace
├── id
├── name
├── icon
├── globalWorkdir
├── toolIds[]
├── profileIds[]
├── favorites[]
└── preferences
```

---

# 9. Workspace não duplica Tools

Uma Tool continua existindo apenas uma vez.

O Workspace apenas referencia essa Tool.

Estrutura correta:

```text
Workspace
└── Tool References
```

e não:

```text
Workspace
└── Tool
    └── Manifest duplicado
```

Exemplo:

```text
Tool:
Curl

Workspaces:

Desenvolvimento
Segurança
DevOps
Redes
```

Existe apenas:

```text
1 Curl Tool
1 Curl Manifest
```

---

# 10. Workspace padrão

O UIGE deve possuir sempre um Workspace padrão.

Exemplo:

```text
Padrão
```

Inicialmente ele pode conter todas as Tools registradas.

```text
Workspace: Padrão

Tools:
Todas
```

Depois o usuário pode criar outros Workspaces.

---

# 11. Workspace por categoria

Exemplo:

```text
Desenvolvimento
```

com:

```text
Git
Docker
Maven
Gradle
Curl
```

---

```text
DevOps
```

com:

```text
Git
Docker
Kubectl
Helm
SSH
Systemctl
```

---

```text
Segurança
```

com:

```text
Nmap
Gobuster
Nikto
Whois
Curl
```

---

# 12. Workspace por projeto

Também deve ser possível organizar por projeto.

Exemplo:

```text
Workspace:
Omny

Global Workdir:
~/Projects/Omny

Tools:

Git
Docker
Maven
Kubectl

Profiles:

Atualizar Omny
Build Backend
Subir Ambiente
Deploy
```

Outro:

```text
Workspace:
UIGE

Global Workdir:
~/Projects/UIGE

Tools:

Git
CMake
Docker

Profiles:

Build UIGE
Atualizar UIGE
Gerar Release
```

Assim o mesmo sistema funciona tanto para categorias quanto para projetos.

---

# 13. Workspace ativo

O UIGE sempre possui:

```text
Active Workspace
```

Internamente:

```text
activeWorkspaceId
```

Exemplo:

```text
activeWorkspaceId = "security"
```

A Home é renderizada utilizando o Workspace ativo.

---

# 14. Global Workdir

Cada Workspace possui seu próprio:

```text
Global Workdir
```

Exemplo:

```text
Desenvolvimento

~/Projects
```

Enquanto:

```text
Segurança

~/Security
```

Ao trocar de Workspace, o Global Workdir também muda.

---

# 15. Tool Workdir

Uma Tool aberta pode utilizar um diretório diferente do Global Workdir.

Exemplo:

```text
Workspace:
Desenvolvimento

Global Workdir:
~/Projects


Git aberto em:

~/Projects/Omny
```

Esse diretório é:

```text
Tool Workdir
```

Ele pertence apenas àquela instância da Tool.

Não altera:

```text
Global Workdir
```

---

# 16. WorkspaceContext

Nome interno recomendado:

```text
WorkspaceContext
```

Pode armazenar informações do Workspace ativo.

Exemplo:

```text
workspaceId
globalWorkdir
preferences
```

---

# 17. ExecutionContext

Uma execução também possui seu próprio contexto.

Nome interno:

```text
ExecutionContext
```

Pode conter:

```text
workdir
environment
variables
permissions
metadata
```

---

# 18. Execution

Uma **Execution** representa a execução de um único Command.

Exemplo:

```text
Tool:
Git

Command:
Pull

Workdir:
~/Projects/UIGE
```

Regra:

```text
1 Execution
=
1 Tool
+
1 Command
+
Parameters
+
ExecutionContext
```

Uma Execution deve registrar:

```text
Tool
Command
Parameters
Workdir
Status
Output
Exit Code
```

---

# 19. Workflow

Um **Workflow** é uma sequência ordenada de Executions.

Exemplo:

```text
1. Git → Checkout main
2. Git → Pull
3. Maven → Package
4. Docker → Compose Up
```

Um Workflow pode futuramente possuir:

```text
condições
tratamento de erro
variáveis
outputs entre etapas
workdir por etapa
retries
fallback
```

---

# 20. WorkflowStep

Cada etapa de um Workflow é representada por:

```text
WorkflowStep
```

Cada WorkflowStep possui:

```text
Tool
Command
Parameters
Workdir opcional
Error Behavior
```

Exemplo:

```text
Step 1

Tool:
Git

Command:
Checkout

Parameters:
branch = main
```

---

# 21. Profile

Um **Profile** representa uma configuração reutilizável de execução.

Ele pode representar:

```text
1 Execution
```

ou:

```text
1 Workflow
```

Todo Profile deve possuir um Manifest principal.

Estrutura:

```text
Profile
├── manifestId
├── name
├── workdirConfiguration
├── openMode
└── action
    ├── Execution
    └── Workflow
```

---

# 22. Relação Profile → Manifest

A relação correta é:

```text
Profile
   ↓
Manifest
```

O Profile referencia o Manifest.

A estrutura NÃO deve ser:

```text
Manifest
└── Profiles
```

O Manifest continua universal.

Exemplo:

```text
Git Manifest
↑
├── Atualizar UIGE
├── Atualizar Omny
├── Criar Branch
└── Preparar Backend
```

---

# 23. Profile x Workspace

É importante não misturar os conceitos.

```text
Workspace
=
organização


Profile
=
execução configurada
```

Exemplo:

```text
Workspace:
Segurança

Tools:
Nmap
Gobuster

Profiles:
Scan Rede Local
Recon Domínio
```

O Profile pode existir independentemente do Workspace.

O Workspace apenas decide se ele aparece naquele ambiente.

---

# 24. Profiles podem aparecer em vários Workspaces

Exemplo:

```text
Profile:
Testar API
```

pode aparecer em:

```text
Desenvolvimento
QA
DevOps
```

sem ser duplicado.

---

# 25. Tipos de Profile

Existem inicialmente:

```text
ExecutionProfile
WorkflowProfile
```

---

# 26. Execution Profile

Executa uma única operação.

Exemplo:

```text
Profile:
Atualizar Projeto

Manifest:
Git

Command:
Pull

Workdir:
~/Projects/UIGE
```

Fluxo:

```text
Profile
 ↓
Execution
 ↓
Command
 ↓
Manifest
```

---

# 27. Workflow Profile

Executa várias operações.

Exemplo:

```text
Profile:
Preparar Backend

Primary Manifest:
Git

Workflow:

1. Git → Checkout main
2. Git → Pull
3. Maven → Package
4. Docker → Compose Up
```

Um Workflow pode utilizar várias Tools.

O Manifest principal representa apenas a origem principal do Profile.

---

# 28. Workdir de Profile

Um Profile pode possuir três modos:

```text
fixed
ask
global
```

## Fixed

Usa sempre um diretório salvo.

```text
~/Projects/UIGE
```

## Ask

Ao abrir:

```text
Escolha um diretório
```

## Global

Utiliza:

```text
Global Workdir
```

do Workspace ativo.

---

# 29. Modo de abertura de Profile

Um Profile pode possuir:

```text
open
confirm
automatic
```

## Open

Carrega a configuração.

Não executa.

---

## Confirm

Carrega a configuração e pede confirmação.

---

## Automatic

Executa imediatamente.

---

# 30. Launcher

Um **Launcher** é uma forma externa de abrir um Profile.

Exemplos:

```text
Desktop Icon
Menu de aplicações
Atalho
```

O Launcher NÃO contém:

```text
Commands
Parameters
Workflow
```

Ele apenas aponta para:

```text
Profile
```

Fluxo:

```text
Launcher
 ↓
Profile
 ↓
Execution / Workflow
```

---

# 31. Desktop Launcher

Implementação:

```text
DesktopLauncher
```

O UIGE pode criar:

```text
.desktop
```

Conceitualmente:

```text
uige profile run <profile-id>
```

Exemplos:

```text
Atualizar UIGE
Atualizar Omny
Scan Rede Local
Subir Backend
```

---

# 32. Favoritos

Favoritos pertencem ao Workspace.

Podem conter:

```text
Tools
Profiles
```

Exemplo:

```text
Workspace:
Desenvolvimento

Favoritos:

Git
Docker
Atualizar Backend
```

Enquanto:

```text
Workspace:
Segurança

Favoritos:

Nmap
Gobuster
Scan Rede
```

---

# 33. Navegação principal

Inicialmente:

```text
Home
Ferramentas
Perfis
Configurações
```

Sidebar possível:

```text
UIGE

⌂ Home
⌘ Ferramentas
▶ Perfis
⚙ Configurações
```

Workspaces não precisam inicialmente de uma tela própria na sidebar.

Eles podem ser controlados pelo seletor da Home.

---

# 34. Seletor de Workspace

No topo:

```text
Workspace:
[ Desenvolvimento ▼ ]
```

Menu:

```text
Desenvolvimento
Segurança
DevOps
Mídia
────────────────────
+ Novo Workspace
Gerenciar Workspaces
```

Trocar Workspace atualiza:

```text
Global Workdir
Tools
Profiles
Favoritos
Preferências visuais
```

---

# 35. Home

A Home deve concentrar as operações comuns.

Exemplo:

```text
┌───────────────────────────────────────────────────────────┐
│ UIGE                                                      │
│                                                           │
│ Workspace: [ Desenvolvimento ▼ ]                          │
│                                                           │
│ 📁 ~/Projects                                [ Workdir ]  │
├───────────────────────────────────────────────────────────┤
│                                                           │
│ Favoritos                                                 │
│                                                           │
│ ★ Git              [▶] [📁] [⋮]                          │
│ ★ Docker           [▶] [📁] [⋮]                          │
│                                                           │
│ Perfis                                                    │
│                                                           │
│ ▶ Atualizar Backend                                       │
│ ▶ Subir Ambiente                                          │
│                                                           │
│ Ferramentas                                               │
│                                                           │
│ Git                [▶] [📁] [⋮]                          │
│ Docker             [▶] [📁] [⋮]                          │
│ Maven              [▶] [📁] [⋮]                          │
│ Curl               [▶] [📁] [⋮]                          │
│                                                           │
│          [+ Manifest]     [+ Perfil]                      │
└───────────────────────────────────────────────────────────┘
```

---

# 36. Funções da Home

A Home deve permitir:

```text
trocar Workspace
alterar Global Workdir
executar Tool rapidamente
abrir Tool em outro Workdir
executar Profile
criar Manifest
criar Profile
gerenciar favoritos
```

---

# 37. Ações da Tool

Cada Tool pode possuir:

```text
Git    [▶] [📁] [⋮]
```

---

## ▶ Quick Execute

Executa rapidamente uma ação utilizando:

```text
Global Workdir
```

---

## 📁 Tool Workdir

Solicita um diretório e abre:

```text
ToolView
```

com:

```text
Tool Workdir
```

---

## ⋮ Menu

Pode possuir:

```text
Adicionar aos favoritos

Adicionar ao Workspace
Remover do Workspace

Criar Profile

Editar Manifest
Histórico do Manifest

Criar atalho
Informações

Remover ferramenta
```

---

# 38. Quick Execute

O **Quick Execute** representa:

```text
1 Tool
1 Command
1 Execution
```

Ele NÃO executa Workflow.

Exemplo:

```text
Git
────────────────────────

Workdir
~/Projects

Ação
[ Pull ▼ ]

Parâmetros
...

[ Abrir ferramenta ]       [ Executar ]
```

Outro:

```text
Nmap
 ↓
Scan
 ↓
Target
 ↓
Execute
```

---

# 39. Tool View

Nome interno:

```text
ToolView
```

Nome visual:

```text
Nome da ferramenta
```

Exemplo:

```text
Git
────────────────────────────────

📁 ~/Projects/Omny       [ Alterar ]

Clone
Status
Pull
Push
Commit
Branch
Checkout
```

A ToolView representa:

```text
Tool
+
Tool Workdir
```

---

# 40. Tela de Ferramentas

Nome visual:

```text
Ferramentas
```

Nome interno possível:

```text
ToolsLibrary
```

Pode possuir:

```text
[ Workspace ] [ Todas ]
```

### Workspace

Mostra apenas Tools do Workspace ativo.

### Todas

Mostra todas as Tools registradas.

Exemplo:

```text
[ Buscar ferramenta... ]

Git
Docker
Maven
Nmap
FFmpeg
Curl
```

---

# 41. Criar Workspace

Ação:

```text
+ Novo Workspace
```

Interface:

```text
Novo Workspace
────────────────────────────

Nome
[ Segurança ]


Workdir padrão
[ ~/Security ] [📁]


Ferramentas

☑ Nmap
☑ Gobuster
☑ Nikto
☑ Curl
☐ Git
☐ Docker


Perfis

☑ Scan Rede Local
☑ Recon Domínio
☐ Atualizar Backend


[ Criar ]
```

---

# 42. Editar Workspace

```text
Workspace — Segurança
────────────────────────────

Nome
[ Segurança ]

Global Workdir
[ ~/Security ]


Ferramentas

Nmap
Gobuster
Nikto
Curl

[ + Adicionar ferramenta ]


Profiles

Scan Rede Local
Recon Domínio

[ + Adicionar perfil ]


[ Salvar ]
```

---

# 43. Remover do Workspace x remover Tool

São operações diferentes.

## Remover do Workspace

```text
Remover do Workspace
```

A Tool continua instalada.

Apenas deixa de aparecer naquele Workspace.

---

## Remover Tool

```text
Remover ferramenta
```

Remove a Tool globalmente.

Se estiver presente em vários Workspaces:

```text
Curl é utilizado por:

Desenvolvimento
Segurança
DevOps
```

o UIGE deve avisar antes da remoção.

---

# 44. Criar Manifest

Botão:

```text
[ + Manifest ]
```

Nome visual:

```text
Nova ferramenta
```

Primeira interface:

```text
Nova ferramenta
────────────────────────

Como deseja criar?

[ Criar manualmente ]

[ Detectar executável ]

[ Importar Manifest ]
```

---

# 45. Manifest Builder

Nome interno:

```text
ManifestBuilder
```

Deve ser reutilizado para:

```text
criar
editar
revisar discovery
revisar importação
```

Interface:

```text
Nova ferramenta
────────────────────────────

Nome
[ Git ]

Executável
[ git ]

Ícone
[ git ]


Comandos

Clone
Pull
Push
Status

[ + Comando ]


                    [ Criar ]
```

---

# 46. Manifest Builder — Simple Mode

Modo padrão:

```text
Simple
```

Mostra:

```text
nome
executável
ícone
commands
parameters
type
flag
required
default
valores principais
```

---

# 47. Manifest Builder — Advanced Mode

Botão:

```text
[ Avançado ]
```

Libera:

```text
requires
conflicts
oneOf
requiresOneOf
when
source
separator
output
risk
permissions
raw JSON
```

---

# 48. Command Builder

```text
Novo comando
────────────────────────

Nome
[ Clone ]

Comando
[ clone ]

Descrição
[ Clonar repositório ]


Parâmetros

Repository
Directory
Depth
Submodules


[ + Parâmetro ]

[ Avançado ]


                     [ Salvar ]
```

---

# 49. Parameter Builder

```text
Novo parâmetro
────────────────────────

Nome
[ Depth ]

ID
[ depth ]

Tipo
[ Integer ▼ ]

Flag
[ --depth ]

Obrigatório
[ ]

Mínimo
[ 1 ]

[ Regras ]


                    [ Salvar ]
```

A tela deve mudar de acordo com:

```text
type
```

Exemplos:

```text
Boolean
String
Integer
URL
File
Directory
Enum
```

---

# 50. Manifest Editor

Criar e editar utilizam o mesmo:

```text
ManifestBuilder
```

Possíveis modos:

```text
Create
Edit
Discovery Review
Import Review
```

---

# 51. Versionamento de Manifest

Primeira versão:

```text
v1
```

Alteração:

```text
v2
```

Depois:

```text
v3
```

Uma versão só deve ser criada caso exista alteração real.

---

# 52. Manifest History

Botão:

```text
[ Histórico ]
```

Tela:

```text
Histórico — Git
────────────────────────

v4   Atual
v3
v2
v1

[ Visualizar ]
[ Restaurar ]
```

Restaurar:

```text
v2
```

quando a atual é:

```text
v4
```

gera:

```text
v5
```

com o conteúdo de `v2`.

Nenhum histórico anterior é destruído.

---

# 53. Criar Profile

Pode ser feito pela Home:

```text
[ + Perfil ]
```

ou pela Tool:

```text
Git
 ↓
⋮
 ↓
Criar Profile
```

Abre:

```text
ProfileBuilder
```

---

# 54. Profile Builder

```text
Novo perfil
──────────────────────────

Nome
[ Atualizar projeto ]


Ferramenta
[ Git ▼ ]


Tipo

(•) Execução

( ) Workflow


                    [ Continuar ]
```

O campo:

```text
Ferramenta
```

internamente representa:

```text
Manifest Reference
```

---

# 55. Execution Profile Builder

```text
Atualizar projeto
──────────────────────────

Ferramenta
Git


Workdir

(•) Salvar diretório
    [ ~/Projects/UIGE ] [📁]

( ) Perguntar ao abrir

( ) Usar Workdir global


Comando
[ Pull ▼ ]


Ao abrir

(•) Apenas abrir

( ) Pedir confirmação

( ) Executar automaticamente


Workspaces

☑ Desenvolvimento
☐ DevOps
☐ Segurança


[ Criar atalho desktop ]


                     [ Salvar ]
```

A seleção de Workspace define apenas onde o Profile aparece.

---

# 56. Workflow Profile Builder

```text
Preparar Backend
────────────────────────────────

Workdir
~/Projects/backend


Etapas

1. Git
   Checkout
   branch: main

2. Git
   Pull

3. Maven
   Package

4. Docker
   Compose Up


[ + Etapa ]


Em caso de erro:
[ Parar Workflow ▼ ]


Ao abrir:
[ Executar automaticamente ▼ ]


Workspaces:

☑ Desenvolvimento
☑ DevOps


                         [ Salvar ]
```

---

# 57. Comportamento de erro

Inicialmente:

```text
Parar Workflow
Continuar
```

Futuramente:

```text
Retry
Fallback
Go To Step
Conditional Step
```

---

# 58. Tela de Profiles

```text
Perfis
```

Exemplo:

```text
[ Workspace ] [ Todos ]

Atualizar UIGE
Atualizar Omny
Scan Local
Deploy Backend
```

Cada Profile pode possuir:

```text
Executar
Editar
Adicionar ao Workspace
Criar atalho
Duplicar
Excluir
```

---

# 59. Profile View

Quando o modo é:

```text
open
```

pode aparecer:

```text
Atualizar UIGE
───────────────────────────

Ferramenta
Git

Workdir
~/Projects/UIGE

Ação
Pull


[ Executar ]
```

---

# 60. Execution View

Uma Execution deve apresentar:

```text
Tool
Command
Workdir
Status
Output
Exit Code
```

Exemplo:

```text
Git — Pull

📁 ~/Projects/UIGE

Executando...

Already up to date.

Exit code: 0
```

---

# 61. Workflow Execution View

```text
Preparar Backend
────────────────────────────────

✓ 1. Git → Checkout main
✓ 2. Git → Pull
▶ 3. Maven → Package
○ 4. Docker → Compose Up

────────────────────────────────

Output

Building project...

[ Cancelar ]
```

Possíveis estados:

```text
Pending
Running
Success
Failed
Skipped
Cancelled
```

---

# 62. Relação dos componentes

```text
UIGE
│
├── Workspace
│   ├── Global Workdir
│   ├── Tool References
│   ├── Profile References
│   ├── Favorites
│   └── Preferences
│
├── Tool
│   └── Manifest
│       ├── Commands
│       │   └── Parameters
│       └── Manifest Versions
│
├── ToolView
│   ├── Tool
│   └── Tool Workdir
│
├── Execution
│   ├── Tool
│   ├── Command
│   ├── Parameters
│   └── ExecutionContext
│
├── Workflow
│   └── WorkflowStep[]
│
├── Profile
│   ├── Manifest Reference
│   ├── Workdir Configuration
│   ├── Open Mode
│   └── Action
│       ├── Execution
│       └── Workflow
│
├── Launcher
│   └── Profile Reference
│
└── Automation
    └── Profile Reference
```

---

# 63. Fluxo — execução rápida

```text
Workspace
 ↓
Home
 ↓
Tool
 ↓
▶
 ↓
Quick Execute
 ↓
Command
 ↓
Execution
```

---

# 64. Fluxo — Tool em diretório específico

```text
Workspace
 ↓
Home
 ↓
Tool
 ↓
📁
 ↓
Selecionar diretório
 ↓
ToolView
 ↓
Command
 ↓
Execution
```

---

# 65. Fluxo — Profile

```text
Workspace
 ↓
Profile
 ↓
Execution / Workflow
 ↓
Tool
 ↓
Manifest
```

---

# 66. Fluxo — Desktop

```text
Desktop Icon
 ↓
DesktopLauncher
 ↓
Profile
 ↓
Execution / Workflow
```

---

# 67. Nomenclatura interna recomendada

```text
Tool

Manifest
ManifestVersion

Command
Parameter

Workspace
WorkspaceManager
WorkspaceContext
WorkspacePreferences

ExecutionContext
Execution

Workflow
WorkflowStep

Profile
ExecutionProfile
WorkflowProfile

Launcher
DesktopLauncher

ManifestBuilder
CommandBuilder
ParameterBuilder
ProfileBuilder
WorkspaceBuilder

VersionManager

ToolView
ProfileView
ExecutionView
WorkflowExecutionView

ToolsLibrary
ProfilesLibrary
```

---

# 68. Nomenclatura visual

A interface não deve expor termos técnicos desnecessariamente.

Internamente:

```text
ManifestBuilder
```

Visualmente:

```text
Nova ferramenta
Editar ferramenta
```

---

Internamente:

```text
ExecutionProfile
WorkflowProfile
```

Visualmente:

```text
Perfil
```

---

Internamente:

```text
ToolView
```

Visualmente:

```text
Git
Docker
Nmap
```

---

Internamente:

```text
DesktopLauncher
```

Visualmente:

```text
Criar atalho
```

---

Internamente:

```text
WorkspaceContext
```

Visualmente:

```text
Workspace
Workdir
```

---

# 69. Workspace temporário — futuro

Poderá existir:

```text
Abrir pasta como Workspace
```

Exemplo:

```text
~/Projects/Teste
```

O UIGE cria um contexto temporário para aquele diretório.

Não é necessário salvar permanentemente.

---

# 70. Automation — futuro

Automation será um conceito independente.

```text
Automation
```

Uma Automation sempre referencia:

```text
Profile
```

Nunca:

```text
Manifest
```

diretamente.

Fluxo:

```text
Automation
 ↓
Profile
 ↓
Execution / Workflow
 ↓
Tool
 ↓
Manifest
```

---

# 71. Exemplo de Automation

Profile:

```text
Atualizar UIGE

Git → Pull
```

Automation:

```text
Atualizar UIGE diariamente

Profile:
Atualizar UIGE

Schedule:
Todos os dias às 08:00
```

Outro:

```text
Profile:
Scan Rede Local

Automation:
A cada 6 horas
```

---

# 72. Tela futura de Automation

```text
Automações
────────────────────────────

Atualizar UIGE
Todo dia às 08:00
Profile: Atualizar UIGE
[ Ativa ]


Backup
Todo domingo às 02:00
Profile: Backup Home
[ Ativa ]


Scan Local
A cada 6 horas
Profile: Scan Rede
[ Desativada ]


[ + Automação ]
```

Nesse momento a navegação poderá ser:

```text
Home
Ferramentas
Perfis
Automações
Configurações
```

---

# 73. Arquitetura resumida

```text
                         MANIFEST
                            │
                           TOOL
                            │
                ┌───────────┴───────────┐
                │                       │
            WORKSPACE               EXECUTION
                │
       ┌────────┼─────────┐
       │        │         │
     Tools    Profiles  Favorites
                │
        ┌───────┴────────┐
        │                │
    Execution         Workflow
        │                │
        └────────┬───────┘
                 │
          ┌──────┴───────┐
          │              │
       Launcher       Automation
          │              │
       Desktop         Schedule
```

---

# 74. Hierarquia de responsabilidade

Uma maneira simples de entender o UIGE é:

```text
Manifest
↓
DEFINE


Tool
↓
DISPONIBILIZA


Workspace
↓
ORGANIZA


Profile
↓
PRECONFIGURA


Execution / Workflow
↓
EXECUTA


Launcher
↓
FACILITA O ACESSO


Automation
↓
AGENDA / AUTOMATIZA
```

---

# 75. Filosofia final

O usuário não deveria precisar entender internamente:

```text
Manifest
ExecutionContext
Parameter Relations
WorkflowStep
Launcher Reference
```

para utilizar o UIGE.

Ele deveria enxergar principalmente:

```text
Workspace
Ferramentas
Ações
Perfis
Workdir
Executar
```

Configurações mais complexas ficam disponíveis através de:

```text
⋮
Editar
Avançado
Builders
Configurações
```

A regra final do projeto é:

> **Manifest define. Tool disponibiliza. Workspace organiza. Profile prepara. Execution executa. Launcher facilita. Automation automatiza.**

E a filosofia visual continua sendo:

> **Simples para executar, poderoso para configurar.**
