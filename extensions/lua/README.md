# Extensões em Lua

## Ideia

A direção futura é fazer de Lua a camada oficial de extensões do Octopus.

A proposta é permitir que outras pessoas criem funcionalidades para o shell sem precisar desenvolver diretamente no núcleo em Rust ou conhecer GTK4.

Lua não seria o núcleo do shell. O núcleo continuaria em Rust; Lua seria a interface de scripting e, futuramente, de extensibilidade.

## O que isso pode permitir

Ainda como possibilidades a explorar:

- widgets e pequenos componentes;
- ações e comandos;
- automações;
- integração com eventos e serviços que o Octopus exponha;
- extensões desenvolvidas pela comunidade.

Esses exemplos são possibilidades, não uma lista fechada de funcionalidades garantidas pela API.

## Relação com o Companion

A intenção é incorporar o runtime Lua ao processo principal, em vez de iniciar um processo separado para cada extensão.

O Companion em Rust seria o host e disponibilizaria uma API controlada para os scripts Lua. As extensões não devem depender de acessar diretamente as estruturas internas do Rust.

Essa separação permite evoluir a implementação interna sem obrigar cada extensão a conhecer os detalhes do núcleo.

## Cuidados para quando isso for implementado

Antes de abrir uma API para terceiros, será necessário definir:

- quais recursos do shell podem ser acessados por extensões;
- como erros de uma extensão são tratados sem derrubar o Companion;
- como permissões e acesso a recursos do sistema funcionam;
- como a API é versionada e como compatibilidade é declarada;
- como extensões são descobertas, carregadas, atualizadas e desativadas.

Esses pontos ainda não têm uma especificação fechada.

## Caminho sugerido

A ideia é começar com o runtime incorporado e uma API pequena, usada por extensões do próprio projeto. Depois de testar os limites e a experiência de desenvolvimento, a API pode ser preparada para contribuições externas.

Não é necessário implementar um ecossistema completo de plugins de uma vez.
