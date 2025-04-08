CREATE TYPE entry_type AS ENUM ('playlist','album');

CREATE TABLE entries(
	id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
	name TEXT NOT NULL,
	image_url TEXT NOT NULL,
	spotify_uri TEXT NOT NULL,
	spotify_id TEXT NOT NULL,
	play_count smallint,
	entry_type entry_type
);
