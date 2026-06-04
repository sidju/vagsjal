BEGIN;

-- Append-only ledger tables recording all character changes.
-- A raise with no corresponding review row is implicitly pending.

CREATE TABLE stat_raise (
stat_raise_id BIGSERIAL PRIMARY KEY NOT NULL,
vampire_id BIGINT NOT NULL,
stat VARCHAR(64) NOT NULL,
increase INT CHECK (increase > 0) NOT NULL,
xp_cost INT CHECK (xp_cost > 0) NOT NULL,
creation_time TIMESTAMPTZ DEFAULT NOW() NOT NULL,

FOREIGN KEY (vampire_id) REFERENCES vampire ON DELETE CASCADE,
FOREIGN KEY (stat) REFERENCES stat
);

CREATE TABLE power_raise (
power_raise_id BIGSERIAL PRIMARY KEY NOT NULL,
vampire_id BIGINT NOT NULL,
power VARCHAR(64) NOT NULL,
xp_cost INT CHECK (xp_cost > 0) NOT NULL,
creation_time TIMESTAMPTZ DEFAULT NOW() NOT NULL,

FOREIGN KEY (vampire_id) REFERENCES vampire ON DELETE CASCADE,
FOREIGN KEY (power) REFERENCES power
);

CREATE TABLE influence_raise (
influence_raise_id BIGSERIAL PRIMARY KEY NOT NULL,
vampire_id BIGINT NOT NULL,
influence VARCHAR(64) NOT NULL,
xp_cost INT CHECK (xp_cost > 0) NOT NULL,
creation_time TIMESTAMPTZ DEFAULT NOW() NOT NULL,

FOREIGN KEY (vampire_id) REFERENCES vampire ON DELETE CASCADE,
FOREIGN KEY (influence) REFERENCES influence
);

CREATE TABLE humanity_change (
humanity_change_id BIGSERIAL PRIMARY KEY NOT NULL,
vampire_id BIGINT NOT NULL,
change INT CHECK (change != 0) NOT NULL,
xp_cost INT CHECK (xp_cost >= 0) NOT NULL, -- Can be 0 for humanity loss
note VARCHAR(256) NOT NULL DEFAULT '',
creation_time TIMESTAMPTZ DEFAULT NOW() NOT NULL,

FOREIGN KEY (vampire_id) REFERENCES vampire ON DELETE CASCADE
);

COMMIT;
