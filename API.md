# StockBridge API — Documentação das rotas

**Base URL:** `http://localhost:3000`
**Formato:** requisições e respostas em JSON (`Content-Type: application/json`).

> **Erros:** quando algo dá errado, o corpo da resposta é **texto puro** (não JSON) com a mensagem, e o status HTTP indica o tipo: `400` (dado inválido / regra de negócio), `404` (não encontrado), `500` (erro interno).
> **CORS** liberado para qualquer origem. **Não há autenticação** nesta versão.

---

## 1. Health check

`GET /api/health` → `200` · `API Rust Operante!`

---

## 2. Produtos

### Criar produto — `POST /api/products`
```bash
curl -X POST http://localhost:3000/api/products \
  -H "Content-Type: application/json" \
  -d '{"name":"Monitor Gamer 24","sku":"MON-GAM-24","cost":150.00,"price":299.90}'
```
`200` → `{ "id": "uuid-do-produto" }`
Erros `400`: nome/SKU vazio · custo ou preço negativo · `Já existe um produto com esse SKU.`

### Listar produtos — `GET /api/products`
`200` →
```json
[
  { "id": "uuid", "name": "Monitor Gamer 24", "sku": "MON-GAM-24",
    "cost": 150.0, "price": 299.9, "totalQuantity": 50 }
]
```
`totalQuantity` = soma do produto em **todas** as estantes. Ordenado por nome.

---

## 3. Depósitos

### Criar depósito — `POST /api/warehouses`
```bash
curl -X POST http://localhost:3000/api/warehouses \
  -H "Content-Type: application/json" -d '{"name":"Galpão Central"}'
```
`200` → `{ "id": "uuid-do-deposito" }` · `400` se o nome for vazio.

### Listar depósitos — `GET /api/warehouses`
`200` → `[ { "id": "uuid", "name": "Galpão Central" } ]`

---

## 4. Localizações (estantes)

### Criar localização — `POST /api/locations`
```bash
curl -X POST http://localhost:3000/api/locations \
  -H "Content-Type: application/json" \
  -d '{"warehouseId":"uuid-do-deposito","sector":"B","rack":"10","level":"2","position":"1","fullCode":"B-10-2-1","qrToken":"QR-B-10-2-1","maxCapacity":100}'
```
`200` → `{ "id": "uuid-da-localizacao" }`
Erros `400`: `maxCapacity` ≤ 0 · `fullCode` ou `qrToken` já existem. O campo `position` é opcional.
`warehouseId` é apenas guardado (não é validado contra a tabela de depósitos).

### Listar localizações — `GET /api/locations`
Filtro opcional por QR Code (para o app ao bipar a estante): `GET /api/locations?qrToken=QR-B-10-2-1`
`200` →
```json
[
  { "id": "uuid", "warehouseId": "uuid", "sector": "B", "rack": "10", "level": "2",
    "position": "1", "fullCode": "B-10-2-1", "qrToken": "QR-B-10-2-1",
    "maxCapacity": 100, "isActive": true, "currentQuantity": 50 }
]
```
`currentQuantity` = quanto já está guardado na estante. Com `qrToken` que não existe, a lista vem vazia (`[]`).

### Estoque de uma localização — `GET /api/locations/{id}/stock`
`200` →
```json
[ { "product_id": "uuid", "product_name": "Monitor Gamer 24", "sku": "MON-GAM-24", "quantity": 50 } ]
```

---

## 5. Movimentações de estoque

### Registrar entrada/saída — `POST /api/movements`
```bash
curl -X POST http://localhost:3000/api/movements \
  -H "Content-Type: application/json" \
  -d '{"productId":"uuid-do-produto","locationId":"uuid-da-localizacao","type":"IN","quantity":50}'
```
- `type`: `"IN"` (entrada) ou `"OUT"` (saída). `quantity`: inteiro **maior que zero**.

`200` → `{ "id": "uuid-da-movimentacao", "new_balance": 50 }` (`new_balance` = saldo do produto **naquela estante**).

Erros `400` (a operação inteira é desfeita, nada é gravado):

| Mensagem | Causa |
|---|---|
| `O campo type deve ser "IN" ou "OUT".` | tipo inválido |
| `A quantidade deve ser maior que zero.` | quantidade ≤ 0 |
| `Localização não encontrada.` | `locationId` inexistente |
| `Produto não encontrado.` | `productId` inexistente |
| `Capacidade excedida. A estante suporta X itens, mas já tem Y guardados.` | entrada estoura o `maxCapacity` |
| `Estoque insuficiente para essa saída.` | saída maior que o saldo |

**Mercado Livre:** depois de gravar, se o produto tiver anúncios vinculados, a API atualiza todos eles em segundo plano com o estoque total do produto. Isso não atrasa nem bloqueia a resposta.

### Histórico — `GET /api/movements`
Mais recente primeiro. Cruza movimentação com custo/preço do produto (para faturamento e lucro no dashboard).
`200` →
```json
[
  { "id": "uuid", "productName": "Monitor Gamer 24", "sku": "MON-GAM-24", "type": "IN",
    "quantity": 50, "cost": 150.0, "price": 299.9, "createdAt": "2026-10-18T22:23:00Z" }
]
```

---

## 6. Mercado Livre

### Status da conexão — `GET /api/meli/status`
`200` → `{ "connected": true, "expiresAt": "2026-11-08T22:00:00Z" }` (ou `{ "connected": false, "expiresAt": null }`).
Nunca devolve os tokens. `expiresAt` é a validade do token atual; a API renova sozinha.

### Callback de autorização — `GET /api/meli/callback?code=TG-...`
Usada uma única vez na conexão (passo a passo no `README.md`). `200` → texto `Integração concluída!...`. Erro `500` com a resposta do Mercado Livre se o código for inválido/expirado.

### Vincular produto a anúncio — `POST /api/meli/links`
```bash
curl -X POST http://localhost:3000/api/meli/links \
  -H "Content-Type: application/json" \
  -d '{"product_id":"uuid-do-produto","meli_item_id":"MLB123456789"}'
```
`200` → `{ "id": "uuid-do-vinculo" }`. Um produto pode ter **vários** anúncios; o mesmo anúncio só pode ser vinculado a **um** produto.
Erros `400`: `meli_item_id` vazio · `Esse anúncio do Mercado Livre já está vinculado a um produto.` · `Produto não encontrado.`

### Listar vínculos — `GET /api/meli/links`
`200` →
```json
[ { "id": "uuid", "productId": "uuid", "meliItemId": "MLB123456789",
    "productName": "Monitor Gamer 24", "sku": "MON-GAM-24", "createdAt": "2026-11-08T21:00:00Z" } ]
```

### Remover vínculo — `DELETE /api/meli/links/{id}`
`204` (sem corpo) · `404` se não existir.
