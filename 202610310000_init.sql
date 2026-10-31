CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    sku TEXT NOT NULL UNIQUE,
    cost DOUBLE PRECISION NOT NULL,
    price DOUBLE PRECISION NOT NULL
);

CREATE TABLE IF NOT EXISTS locations (
    id TEXT PRIMARY KEY,
    "warehouseId" TEXT NOT NULL,
    sector TEXT NOT NULL,
    rack TEXT NOT NULL,
    level TEXT NOT NULL,
    position TEXT NOT NULL,
    "fullCode" TEXT NOT NULL UNIQUE,
    "qrToken" TEXT NOT NULL UNIQUE,
    "maxCapacity" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS stocks (
    id TEXT PRIMARY KEY,
    "productId" TEXT NOT NULL REFERENCES products(id),
    "locationId" TEXT NOT NULL REFERENCES locations(id),
    quantity INTEGER NOT NULL,
    UNIQUE("productId", "locationId")
);

CREATE TABLE IF NOT EXISTS movements (
    id TEXT PRIMARY KEY,
    "productId" TEXT NOT NULL REFERENCES products(id),
    "locationId" TEXT NOT NULL REFERENCES locations(id),
    type TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS meli_credentials (
    id TEXT PRIMARY KEY,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS meli_links (
    id TEXT PRIMARY KEY,
    "productId" TEXT NOT NULL REFERENCES products(id),
    "meliItemId" TEXT NOT NULL UNIQUE,
    "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
