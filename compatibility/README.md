# Escopo de compatibilidade

## Prioridade do projeto

O Octopus é um projeto pessoal desenvolvido para funcionar muito bem na combinação de tecnologias escolhida para ele. A prioridade é a qualidade no ambiente principal, não oferecer suporte universal a todos os compositores e desktops Linux.

A modularidade interna não obriga o projeto a suportar GNOME, KDE, Hyprland ou qualquer outra combinação que não faça parte do escopo definido.

## Dois objetivos diferentes

- **Ambiente principal:** funcionar muito bem na configuração usada no desenvolvimento e no dia a dia.
- **Distribuição:** poder funcionar em outras instalações que atendam aos requisitos que o projeto declarar.

O segundo objetivo não precisa transformar o primeiro em uma camada de abstração genérica.

## Escolhas conscientes

As tecnologias são escolhidas porque fazem sentido para o projeto e para quem o desenvolve. Se uma dependência ou integração deixar de atender às necessidades do Octopus, ela pode ser reavaliada.

O projeto pode trocar ou combinar linguagens e bibliotecas sem assumir que precisa ser compatível com qualquer ambiente possível.

## Regra prática

Não adicionar suporte ou abstrações só para aumentar uma lista de compatibilidade. Fazer isso quando existir uma necessidade real do projeto ou dos ambientes que ele decidiu atender.
