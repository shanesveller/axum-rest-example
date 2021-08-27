CREATE TABLE links (
  id uuid DEFAULT uuid_generate_v4 () PRIMARY KEY,
  destination text NOT NULL
);

CREATE UNIQUE INDEX links_destination ON links (destination);
