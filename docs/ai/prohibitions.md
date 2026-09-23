# O que a IA não deve fazer

A IA não deve:

- apagar funcionalidade para “simplificar” sem autorização;
- alterar comportamento fora do escopo solicitado;
- inventar requisito que não está documentado;
- duplicar uma regra já existente em outro lugar;
- criar classes, serviços, interfaces ou camadas sem necessidade concreta;
- introduzir dependência sem justificar seu uso;
- mudar nomes canônicos apenas por preferência;
- misturar regra de produto com detalhe de framework;
- colocar preferências pessoais do usuário dentro de Manifest;
- duplicar Tool/Manifest para cada Workspace;
- tornar Launcher ou Automation donos da configuração que pertence ao Profile;
- apagar histórico de Manifest ou arquivos anteriores de `/changes/`;
- marcar tarefa como concluída sem validar quando houver validação possível;
- esconder erro, warning ou falha de teste relevante;
- modificar `LICENSE`, política de segurança ou decisões arquiteturais centrais incidentalmente;
- escolher stack definitiva sem uma decisão explícita/ADR quando ainda estiver em aberto.
