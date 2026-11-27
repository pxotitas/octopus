# Linguagens e responsabilidades

A ideia não é escolher uma única linguagem para tudo. Cada tecnologia deve entrar quando fizer sentido para a responsabilidade que vai assumir.

## Rust + GTK4: núcleo

Rust continua sendo a base do Companion e da interface GTK4.

É onde ficam a coordenação geral do shell, o estado compartilhado, os serviços e a lógica que precisa conversar diretamente com a interface.

A arquitetura atual já é organizada em módulos dentro do Companion. A intenção é preservar essa organização, em vez de transformar cada funcionalidade em um executável independente.

## Lua: scripting incorporado

A decisão é substituir os scripts Python atuais por Lua.

Além de ser uma linguagem que já faz parte da experiência de desenvolvimento do projeto, Lua tem uma característica importante para esta arquitetura: pode ser incorporada ao processo que hospeda o shell.

A migração deve ser gradual. Não existe, neste momento, uma lista fechada de scripts nem uma ordem definida para a substituição.

Também existe a intenção de usar Lua futuramente como a camada de extensões do Octopus. Isso está descrito em [Extensões em Lua](../extensions/lua/README.md).

## C: integração nativa quando fizer sentido

C fica disponível para partes específicas que realmente se beneficiem de integração nativa, acesso a APIs ou controle de baixo nível.

Ainda não foram escolhidos os scripts ou módulos que serão substituídos por C. Essa avaliação continua em aberto.

A escolha não deve ser feita só porque C é mais baixo nível: cada caso precisa ter uma justificativa técnica e considerar manutenção, segurança e integração com o restante do Companion.

## Um processo, um executável

Uma preferência importante discutida para essa arquitetura é evitar criar vários processos auxiliares para funcionalidades que podem viver dentro do Companion.

A intenção é manter Rust, Lua e eventuais partes em C integrados ao binário principal e ao mesmo processo. Os detalhes de compilação e interoperabilidade ainda precisam ser definidos conforme cada integração.

Isso não significa que todo processo externo esteja proibido: ferramentas do sistema e serviços externos dos quais o shell depende continuam sendo integrações normais. O objetivo é não fragmentar o próprio código do Octopus sem necessidade.

## Critérios para escolher uma linguagem

Antes de introduzir ou migrar uma parte, considerar:

- responsabilidade e complexidade daquele componente;
- desempenho necessário na prática;
- legibilidade e facilidade de manutenção por outras pessoas;
- maturidade das bibliotecas e ferramentas;
- integração com Rust e com as APIs do sistema;
- dependências e licenças;
- custo de compilar, distribuir, depurar e testar.

A arquitetura deve escolher a linguagem conforme o problema, não o contrário.
