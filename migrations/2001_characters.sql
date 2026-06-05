BEGIN;

CREATE TABLE vampire (
vampire_id BIGSERIAL PRIMARY KEY,
-- Meta information --
user_id BIGINT NOT NULL,
status VARCHAR(16) NOT NULL DEFAULT 'draft',

-- Character descriptive --
name VARCHAR(64) NOT NULL,
apparent_age INT NOT NULL, --in years--
date_embraced DATE NOT NULL CHECK (date_embraced <= CURRENT_DATE),
torpor_time INTERVAL NOT NULL,
clan_id BIGINT NOT NULL,
covenant_id BIGINT,
character_description_url TEXT,
public_knowledge TEXT NOT NULL DEFAULT '',
home_domain VARCHAR(128) NOT NULL DEFAULT '',
known_age VARCHAR(64) NOT NULL DEFAULT '',

FOREIGN KEY (user_id) REFERENCES app_user,
FOREIGN KEY (clan_id) REFERENCES clan,
FOREIGN KEY (covenant_id) REFERENCES covenant (covenant_id),
FOREIGN KEY (status) REFERENCES character_status
);

-- The table all XP comes from, insert only --
CREATE TABLE xp (
xp_id BIGSERIAL PRIMARY KEY NOT NULL,
vampire_id BIGINT NOT NULL,
amount INT CHECK (amount > 0) NOT NULL,

FOREIGN KEY (vampire_id) REFERENCES vampire ON DELETE CASCADE
);

CREATE FUNCTION grant_vampire_base_xp()
RETURNS TRIGGER AS $$
DECLARE
  years_awake INT;
BEGIN
  -- Prevent reverting from active/inactive back to draft
  IF TG_OP = 'UPDATE' AND OLD.status IN ('active', 'inactive') AND NEW.status = 'draft' THEN
    RAISE EXCEPTION 'Cannot change an active or inactive character back to draft';
  END IF;

  -- Grant starting XP when a character becomes active or inactive
  -- (from draft on UPDATE, or directly on INSERT)
  IF NEW.status IN ('active', 'inactive') AND (TG_OP = 'INSERT' OR OLD.status = 'draft') THEN
    years_awake := FLOOR(
      EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - NEW.date_embraced::TIMESTAMPTZ - NEW.torpor_time))
      / (86400.0 * 365.25)
    )::INT;
    INSERT INTO xp (vampire_id, amount) VALUES (NEW.vampire_id, 24 + 2 * years_awake);
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_vampire_grant_base_xp
  AFTER INSERT OR UPDATE OF status ON vampire
  FOR EACH ROW
  EXECUTE FUNCTION grant_vampire_base_xp();

COMMIT;
