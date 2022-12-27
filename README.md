# iSmallTalk
p2p chat app built in Rust with GTK3 Gui

## To Do Development
* ### HostClients 
- HOST: envia todos os clients ao novo client
- NOVO CLIENT: adiciona todos os clients (incluindo o proprio host) a lista de clients

* ### AddClients
- HOST: envia novo client aos demais clients
- TODOS OS DEMAIS CLIENTS: adicionar novo client na lista de clients

### Other
* multiple senders (tx, rx) para cada funcionalidade
* Remoção dos clients ao ter erro
* Remoção dos clients ao signal de LeaveClient
* Chat history
* Enviar mensagem por signal Message
* Algoritmo de ofuscação do IP