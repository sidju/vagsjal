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

COMMIT;
