BEGIN;

CREATE TYPE approval_state_t AS ENUM('approved', 'denied');

-- raise_id is the primary key: one review per raise, no re-reviews.
-- If a review was made in error, a correcting raise can be added instead.

CREATE TABLE stat_raise_review (
stat_raise_id BIGINT PRIMARY KEY NOT NULL,
state approval_state_t NOT NULL,
reviewer_id BIGINT NOT NULL,
review_time TIMESTAMPTZ DEFAULT NOW() NOT NULL,

FOREIGN KEY (stat_raise_id) REFERENCES stat_raise,
FOREIGN KEY (reviewer_id) REFERENCES app_user
);

CREATE TABLE power_raise_review (
power_raise_id BIGINT PRIMARY KEY NOT NULL,
state approval_state_t NOT NULL,
reviewer_id BIGINT NOT NULL,
review_time TIMESTAMPTZ DEFAULT NOW() NOT NULL,

FOREIGN KEY (power_raise_id) REFERENCES power_raise,
FOREIGN KEY (reviewer_id) REFERENCES app_user
);

CREATE TABLE influence_raise_review (
influence_raise_id BIGINT PRIMARY KEY NOT NULL,
state approval_state_t NOT NULL,
reviewer_id BIGINT NOT NULL,
review_time TIMESTAMPTZ DEFAULT NOW() NOT NULL,

FOREIGN KEY (influence_raise_id) REFERENCES influence_raise,
FOREIGN KEY (reviewer_id) REFERENCES app_user
);

CREATE TABLE humanity_change_review (
humanity_change_id BIGINT PRIMARY KEY NOT NULL,
state approval_state_t NOT NULL,
reviewer_id BIGINT NOT NULL,
review_time TIMESTAMPTZ DEFAULT NOW() NOT NULL,

FOREIGN KEY (humanity_change_id) REFERENCES humanity_change,
FOREIGN KEY (reviewer_id) REFERENCES app_user
);

-- Constraint trigger: prevent a second pending power raise for the same power.
-- A pending raise is one where no review row exists.
-- Counts AFTER insert so the just-inserted row is included, then rejects if > 1.
CREATE FUNCTION check_pending_power_raise() RETURNS TRIGGER AS $$
BEGIN
  IF (
    SELECT COUNT(*) FROM power_raise pr
    LEFT JOIN power_raise_review prr USING (power_raise_id)
    WHERE pr.vampire_id = NEW.vampire_id
      AND pr.power = NEW.power
      AND prr.power_raise_id IS NULL
  ) > 1 THEN
    RAISE EXCEPTION 'Det finns redan en pågående höjning för kraften %', NEW.power
      USING ERRCODE = '23514';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE CONSTRAINT TRIGGER trg_check_pending_power_raise
  AFTER INSERT ON power_raise
  DEFERRABLE INITIALLY IMMEDIATE
  FOR EACH ROW
  EXECUTE FUNCTION check_pending_power_raise();

COMMIT;
