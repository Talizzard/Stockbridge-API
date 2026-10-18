# 📦 StockBridge API (Versão Rust)

Documentação oficial das rotas do backend em Rust (Axum + SQLx + PostgreSQL).

**Base URL:** `http://localhost:3000`

---

## 🟢 1. Health Check
Verifica se a API está no ar e respondendo.

**GET** `/api/health`

**Comando de Teste:**
```bash
curl http://localhost:3000/api/health
```

**Saída Esperada (200 OK):**
```text
API Rust Operante!
```

---

## 📍 2. Criar Localização
Cria um novo espaço físico no depósito para armazenar produtos. O banco de dados gera um UUID automaticamente para a nova localização.

**POST** `/api/locations`

**Comando de Teste:**
*(Substitua o `warehouseId` pelo ID real do seu depósito no banco de dados)*
```bash
curl -X POST http://localhost:3000/api/locations \
-H "Content-Type: application/json" \
-d '{"warehouseId":"621644f1-e70a-46db-a72a-5f3c7fb01a24","sector":"B","rack":"10","level":"2","position":"A","fullCode":"B-10-2-A","qrToken":"TESTE_RUST_001"}'
```

**Saída Esperada (200 OK):**
```json
{
  "id": "594be105-9c1b-43cf-a011-d9d4b2524d69"
}
```

---

## 📦 3. Movimentação de Estoque (Entrada e Saída)
Registra a entrada (`IN`) ou saída (`OUT`) de produtos em uma localização específica. A API possui trava de segurança transacional: se uma saída for maior que o estoque atual, a operação é abortada no banco de dados.

**POST** `/api/movements`

### Exemplo A: Entrada de Estoque (IN)
Adicionando 50 unidades do produto.

**Comando de Teste:**
```bash
curl -X POST http://localhost:3000/api/movements \
-H "Content-Type: application/json" \
-d '{"productId":"prod-001","locationId":"594be105-9c1b-43cf-a011-d9d4b2524d69","type":"IN","quantity":50}'
```

**Saída Esperada (200 OK):**
```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "new_balance": 50
}
```

### Exemplo B: Saída de Estoque (OUT) Bem-sucedida
Retirando 20 unidades de um local que possui saldo suficiente.

**Comando de Teste:**
```bash
curl -X POST http://localhost:3000/api/movements \
-H "Content-Type: application/json" \
-d '{"productId":"prod-001","locationId":"594be105-9c1b-43cf-a011-d9d4b2524d69","type":"OUT","quantity":20}'
```

**Saída Esperada (200 OK):**
```json
{
  "id": "f0e9d8c7-b6a5-4321-dcba-0987654321fe",
  "new_balance": 30
}
```

### Exemplo C: Erro de Estoque Negativo (Bloqueio)
Tentando retirar 100 unidades quando o saldo é apenas 30.

**Comando de Teste:**
```bash
curl -X POST http://localhost:3000/api/movements \
-H "Content-Type: application/json" \
-d '{"productId":"prod-001","locationId":"594be105-9c1b-43cf-a011-d9d4b2524d69","type":"OUT","quantity":100}'
```

**Saída Esperada (400 Bad Request):**
```text
Estoque insuficiente para essa saída.
```

---
---

## 🚀 Como rodar o Banco de Dados (Podman)
Se for a primeira vez rodando em um PC novo, suba o contêiner do PostgreSQL:
```bash
podman run --name stockbridge-db -e POSTGRES_USER=admin -e POSTGRES_PASSWORD=senha_secreta -e POSTGRES_DB=stockbridge -p 5432:5432 -d postgres
```

*(Nota: para ligar e desligar no dia a dia, use `podman start stockbridge-db` e `podman stop stockbridge-db`)*

## ⚙️ Variáveis de Ambiente
Crie um arquivo `.env` na raiz com:
`DATABASE_URL=postgres://admin:senha_secreta@localhost:5432/stockbridge`
