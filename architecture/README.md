# Arquitetura do Octopus

Este diretório guarda as decisões e direções arquiteturais do Octopus. A ideia é registrar o que já foi conversado sem transformar toda possibilidade em obrigação.

## Visão geral

O Octopus é, antes de tudo, um ambiente pessoal que precisa funcionar muito bem na combinação de tecnologias escolhida para ele. A modularidade do código é importante, mas isso não significa que o projeto precise tentar suportar qualquer compositor, barra ou desktop environment.

A arquitetura deve continuar organizada por responsabilidades, sem criar abstrações ou processos separados só por princípio.

## Direção de tecnologias

A direção discutida até agora é:

- **Rust + GTK4:** núcleo do Companion, interface, estado, serviços e coordenação do shell.
- **C:** disponível para partes em que integração nativa ou controle de baixo nível tragam uma vantagem concreta. Ainda não foram escolhidos os módulos que serão escritos ou reescritos em C.
- **Lua:** linguagem de scripting incorporada ao processo principal. A intenção é substituir os scripts Python atuais por Lua e, futuramente, usar Lua como base para extensões do shell.

A intenção é manter o Companion como um único processo e evitar multiplicar processos auxiliares sem necessidade.

## Decisões e ideias futuras

Nem tudo aqui já está implementado. Os documentos desta pasta distinguem decisões atuais, direções desejadas e pontos que ainda precisam de projeto técnico.

- [Linguagens e integração](linguagens.md)
- [Extensões em Lua](../extensions/lua/README.md)
- [Escopo de compatibilidade](../compatibility/README.md)
