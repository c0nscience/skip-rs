ALTER TABLE categories ADD COLUMN visible boolean NOT NULL DEFAULT false;
ALTER TABLE entries ADD COLUMN visible boolean NOT NULL DEFAULT false;
