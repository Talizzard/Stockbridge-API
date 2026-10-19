```markdown
# StockBridge API (Versão Rust)

Documentação oficial das rotas do backend em Rust (Axum + SQLx + PostgreSQL).

**Base URL:** `http://localhost:3000`

---

## 1. Health Check
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

## 2. Produtos

Gerencia o catálogo de produtos com informações financeiras essenciais para cálculos de ROI, margem e lucro no dashboard.

### Criar Produto

**POST** `/api/products`

**Comando de Teste:**

```bash
curl -X POST http://localhost:3000/api/products \
-H "Content-Type: application/json" \
-d '{"name":"Monitor Gamer 24","sku":"MON-GAM-24","cost":150.00,"price":299.90}'

```

**Saída Esperada (200 OK):**

```json
{
  "id": "uuid-do-produto"
}

```

---

## 3. Localizações

Gerencia o espaço físico do depósito e a capacidade máxima das estantes.

### Criar Localização

Cria um novo espaço físico no depósito para armazenar produtos, definindo o limite de capacidade.

**POST** `/api/locations`

**Comando de Teste:**
*(Substitua o `warehouseId` pelo ID real do seu depósito no banco de dados)*

```bash
curl -X POST http://localhost:3000/api/locations \
-H "Content-Type: application/json" \
-d '{"warehouseId":"621644f1-e70a-46db-a72a-5f3c7fb01a24","sector":"B","rack":"10","level":"2","position":"A","fullCode":"B-10-2-A","qrToken":"TESTE_RUST_001","maxCapacity":100}'

```

**Saída Esperada (200 OK):**

```json
{
  "id": "594be105-9c1b-43cf-a011-d9d4b2524d69"
}

```

### Consultar Estoque da Localização

Retorna todos os produtos armazenados em uma estante específica, com suas respectivas quantidades.

**GET** `/api/locations/{id}/stock`

**Comando de Teste:**

```bash
curl -X GET http://localhost:3000/api/locations/594be105-9c1b-43cf-a011-d9d4b2524d69/stock

```

**Saída Esperada (200 OK):**

```json
[
  {
    "product_id": "uuid-do-produto",
    "product_name": "Monitor Gamer 24",
    "sku": "MON-GAM-24",
    "quantity": 50
  }
]

```

---

## 4. Movimentação de Estoque e Histórico

Gerencia e lista as entradas (`IN`) ou saídas (`OUT`) de produtos. A API possui duas travas de segurança transacional no banco de dados:

1. Impede a entrada se a quantidade ultrapassar o `maxCapacity` da estante.
2. Impede a saída se a quantidade for maior que o estoque atual.

### Histórico de Movimentações (Dashboard)

Retorna a linha do tempo completa, cruzando os dados de movimentação com os dados financeiros dos produtos para cálculos de faturamento e lucro. Ordenado do mais recente para o mais antigo.

**GET** `/api/movements`

**Comando de Teste:**

```bash
curl -X GET http://localhost:3000/api/movements

```

**Saída Esperada (200 OK):**

```json
[
  {
    "id": "uuid-da-movimentacao",
    "productName": "Monitor Gamer 24",
    "sku": "MON-GAM-24",
    "type": "IN",
    "quantity": 50,
    "cost": 150.0,
    "price": 299.9,
    "createdAt": "2026-10-18T22:23:00Z"
  }
]

```

### Exemplo A: Entrada de Estoque (IN) Bem-sucedida

Adicionando 50 unidades do produto (respeitando o limite de capacidade).

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

### Exemplo D: Erro de Capacidade Excedida (Bloqueio)

Tentando guardar itens além do limite `maxCapacity` da localização.

**Comando de Teste:**

```bash
curl -X POST http://localhost:3000/api/movements \
-H "Content-Type: application/json" \
-d '{"productId":"prod-001","locationId":"594be105-9c1b-43cf-a011-d9d4b2524d69","type":"IN","quantity":500}'

```

**Saída Esperada (400 Bad Request):**

```text
Capacidade excedida. A estante suporta 100 itens, mas já tem X guardados.

```

---

---

## Como rodar o Banco de Dados (Podman)

Se for a primeira vez rodando em um PC novo, suba o contêiner do PostgreSQL:

```bash
podman run --name stockbridge-db -e POSTGRES_USER=admin -e POSTGRES_PASSWORD=senha_secreta -e POSTGRES_DB=stockbridge -p 5432:5432 -d postgres

```

*(Nota: para ligar e desligar no dia a dia, use `podman start stockbridge-db` e `podman stop stockbridge-db`)*

## Variáveis de Ambiente

Crie um arquivo `.env` na raiz com:
`DATABASE_URL=postgres://admin:senha_secreta@localhost:5432/stockbridge`
