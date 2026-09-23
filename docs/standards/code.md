# Padrões de implementação

## Princípios

- simplicidade antes de abstração;
- baixo acoplamento;
- alta coesão;
- dependências explícitas;
- funções e módulos com responsabilidade clara;
- evitar duplicação de regra de negócio;
- tratar erros na camada responsável;
- não esconder efeitos colaterais importantes.

## Proibições

- não criar abstração sem uso concreto;
- não adicionar dependência para resolver problema trivial;
- não misturar UI com regras centrais do domínio;
- não acoplar Manifest a preferências do usuário;
- não duplicar Tool/Manifest por Workspace;
- não executar comandos montando strings inseguras quando uma API de argumentos estruturados estiver disponível.

## Tecnologia

Padrões específicos de linguagem/framework só devem existir após a tecnologia ser oficialmente adotada e registrada em ADR.
