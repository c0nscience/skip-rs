ALTER TABLE entries 
DROP CONSTRAINT entries_category_id_fkey;

ALTER TABLE entries 
DROP COLUMN category_iD;

DROP TABLE categories;
DROP TYPE category_type;

