BEGIN;

CREATE TABLE vampire (
vampire_id BIGSERIAL PRIMARY KEY,
-- Meta information --
user_id BIGINT NOT NULL,
active BOOL NOT NULL,

-- Character descriptive --
name VARCHAR(64) NOT NULL,
apparent_age INT NOT NULL, --in years--
date_embraced DATE NOT NULL,
torpor_time INTERVAL NOT NULL,
clan_id BIGINT NOT NULL,

FOREIGN KEY (user_id) REFERENCES app_user,
FOREIGN KEY (clan_id) REFERENCES clan
);

-- The table all XP comes from, insert only --
CREATE TABLE xp (
xp_id BIGSERIAL PRIMARY KEY NOT NULL,
vampire_id BIGINT NOT NULL,
amount INT CHECK (amount > 0) NOT NULL,

FOREIGN KEY (vampire_id) REFERENCES vampire ON DELETE CASCADE
);

CREATE FUNCTION insert_vampire_base_xp()
RETURNS TRIGGER AS $$
DECLARE
  years_awake INT;
BEGIN
  years_awake := FLOOR(
    EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - NEW.date_embraced::TIMESTAMPTZ - NEW.torpor_time))
    / (86400.0 * 365.25)
  )::INT;
  INSERT INTO xp (vampire_id, amount) VALUES (NEW.vampire_id, 24 + 2 * years_awake);
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_vampire_insert_base_xp
  AFTER INSERT ON vampire
  FOR EACH ROW
  EXECUTE FUNCTION insert_vampire_base_xp();

COMMIT;
