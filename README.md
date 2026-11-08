# StockBridge API

Backend (API REST) de controle de estoque de depósito (WMS) com sincronização automática com o Mercado Livre.
Feito em **Rust** (Axum + SQLx + PostgreSQL). A documentação de todas as rotas está em [`API.md`](API.md).

---

## 1. O que instalar (uma vez só)

| Programa | Para quê | Onde pegar |
|---|---|---|
| **Git** | baixar o projeto | https://git-scm.com |
| **Rust (Cargo)** | compilar e rodar a API | https://rustup.rs |
| **Docker Desktop** (ou **Podman** no Linux) | rodar o banco PostgreSQL sem instalar nada | https://www.docker.com |
| **Compilador C** | o Rust precisa dele para compilar algumas bibliotecas | veja abaixo |

Compilador C por sistema:
- **Windows:** instale o *Visual Studio Build Tools* com a carga "Desenvolvimento para desktop com C++".
- **Ubuntu/Mint:** `sudo apt install build-essential pkg-config`
- **Fedora:** `sudo dnf install gcc`

> No Windows, **abra o Docker Desktop e espere o ícone ficar verde** antes de rodar qualquer comando `docker`.

---

## 2. Rodando pela primeira vez (passo a passo)

Abra um terminal (PowerShell no Windows) e rode **na ordem**:

**Passo 1 — baixar o projeto**
```bash
git clone LINK_DO_REPOSITORIO
cd stockbridge
```

**Passo 2 — criar o arquivo de configuração `.env`**
```bash
# Linux / Mac
cp .env.example .env

# Windows (PowerShell)
Copy-Item .env.example .env
```
Para só testar a API, não precisa mexer em nada no `.env`. As chaves `MELI_*` só importam na hora de conectar ao Mercado Livre (seção 5).

**Passo 3 — subir o banco de dados**
```bash
docker compose up -d
```
(Com Podman: `podman-compose up -d`.)
Na **primeira vez**, o banco já nasce com todas as tabelas, porque o Postgres executa os arquivos da pasta `migrations/`. Isso é necessário porque o Rust confere as queries SQL **no momento da compilação**, então as tabelas precisam existir antes do `cargo run`. Espere uns 10 segundos para o banco terminar de iniciar.

**Passo 4 — rodar a API**
```bash
cargo run
```
A primeira compilação demora alguns minutos (ele baixa e compila as bibliotecas). Quando aparecer
`🚀 Servidor rodando com alta performance na porta 3000!`, está pronto.

**Passo 5 — testar** (em outro terminal)
```bash
curl http://localhost:3000/api/health
```
Resposta esperada: `API Rust Operante!`

---

## 3. Se algo der errado

| Sintoma | O que fazer |
|---|---|
| `relation "xxx" does not exist` ao compilar | O banco subiu sem as tabelas (acontece se o volume já existia vazio). Rode `docker compose down -v` e depois `docker compose up -d`. **Atenção:** o `-v` apaga os dados do banco. |
| `failed to connect to the docker API` | O Docker Desktop não está aberto. Abra, espere ficar verde e rode de novo. |
| `container name "/stockbridge-db" is already in use` | Existe um container antigo com esse nome: `docker rm -f stockbridge-db` e suba de novo. |
| `Address already in use` ao rodar a API | Já tem uma API rodando na porta 3000. Linux: `fuser -k 3000/tcp`. Windows: feche o outro terminal com `cargo run` (Ctrl+C). |
| `DATABASE_URL não configurada` | Você esqueceu o Passo 2 (criar o `.env`). |
| Usando WSL/Ubuntu no Windows e o navegador não acessa `localhost:3000` | A API já escuta em `0.0.0.0`. Confira se o Docker Desktop tem a integração com o WSL ligada (Settings → Resources → WSL Integration). |

**Regra das migrations:** nunca edite um arquivo de `migrations/` que já rodou em algum banco. Para mudar a estrutura, crie um arquivo novo, por exemplo `migrations/202611150000_adiciona_coluna.sql`.

---

## 4. Gerar o executável final (release)

```bash
cargo build --release
```
O executável fica em `target/release/stockbridge` (Linux/Mac) ou `target\release\stockbridge.exe` (Windows).

- As **migrations vão embutidas** no executável: ao abrir, ele cria/atualiza as tabelas sozinho.
- O **PostgreSQL não vai dentro do executável**. Quem for usar precisa de um banco acessível (o `docker compose up -d` deste projeto, ou um Postgres na nuvem como Neon/Supabase).
- O executável lê o `.env` da **pasta onde ele é executado**. Coloque o `.env` ao lado dele.
- Para **compilar**, o banco precisa estar rodando e com as tabelas (mesmo motivo do Passo 3).

---

## 5. Conectar ao Mercado Livre (uma vez)

A conta do Mercado Livre usada precisa ser a **dona dos anúncios** e ter o app criado no DevCenter, com a URI de redirecionamento `https://google.com` (o Mercado Livre exige `https`, por isso usamos esse truque). Preencha `MELI_APP_ID` e `MELI_SECRET_KEY` no `.env` e **reinicie a API**.

1. Monte o link, trocando `SEU_APP_ID` pelo valor do `.env`, e abra no navegador:
   `https://auth.mercadolivre.com.br/authorization?response_type=code&client_id=SEU_APP_ID&redirect_uri=https://google.com`
2. Faça login na conta do Mercado Livre e clique em **Permitir**.
3. Você cai na página do Google. Olhe a **barra de endereços**: a URL tem `?code=TG-...`. Copie só o código (do `TG-` até o fim, sem o `&` e o que vier depois).
4. Em outra aba, abra: `http://localhost:3000/api/meli/callback?code=COLE_O_CODIGO_AQUI`
5. Deve aparecer: **"Integração concluída!"**. Confirme em `GET /api/meli/status` (`"connected": true`).

Cuidados:
- O código vale **uma única vez** e expira em poucos minutos. Se der `invalid_grant`, refaça os passos 1 a 4 com um código novo, e rápido.
- A API renova o token sozinha (a cada 6 horas) quando precisa.

**Como a sincronização funciona:** depois de vincular um produto a um ou mais anúncios (`POST /api/meli/links`), toda entrada ou saída de estoque atualiza os anúncios no Mercado Livre com o **estoque total do produto** (somando todas as estantes). Isso roda em segundo plano: se o Mercado Livre falhar, a movimentação no depósito continua valendo, e o erro aparece só no terminal da API.

---

## 6. Estrutura do projeto

```text
stockbridge/
├── .env.example          # modelo do .env (o .env real NÃO vai para o Git)
├── Cargo.toml / Cargo.lock
├── docker-compose.yml    # sobe o PostgreSQL
├── API.md                # documentação das rotas (para o frontend)
├── migrations/           # SQL que cria as tabelas (roda sozinho)
└── src/
    ├── main.rs           # liga o banco, o CORS e as rotas
    ├── models/stock.rs   # estruturas de entrada/saída (JSON)
    ├── handlers/         # uma rota por função (products, warehouses, locations, movements, meli)
    └── services/meli.rs  # comunicação com a API do Mercado Livre (token + sincronização)
```
