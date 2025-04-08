CREATE TYPE category_type AS ENUM ('music','audiobook');

CREATE TABLE categories(
	id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
	name TEXT NOT NULL,
	image_url TEXT NOT NULL,
	category_type category_type NOT NULL DEFAULT 'audiobook'
);

ALTER TABLE entries ADD COLUMN category_id UUID REFERENCES categories(id);
