CREATE TABLE links (
  id uuid DEFAULT uuid_generate_v4 () PRIMARY KEY,
  hash text NOT NULL,
  destination text NOT NULL
);

CREATE UNIQUE INDEX links_destination ON links (destination);
CREATE UNIQUE INDEX links_hash ON links (hash);
