-- Add this to your Postgres schema for wallet_roles
CREATE TABLE IF NOT EXISTS wallet_roles (
  wallet TEXT NOT NULL REFERENCES wallets(wallet),
  role TEXT NOT NULL,
  PRIMARY KEY (wallet, role)
);
