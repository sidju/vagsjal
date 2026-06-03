BEGIN;

CREATE TABLE stat (
stat_id BIGSERIAL UNIQUE NOT NULL,
name VARCHAR(64) PRIMARY KEY,
description VARCHAR NOT NULL DEFAULT ''
);

CREATE TABLE power (
power_id BIGSERIAL UNIQUE NOT NULL,
name VARCHAR(64) PRIMARY KEY,
description VARCHAR NOT NULL DEFAULT ''
);

CREATE TABLE influence (
influence_id BIGSERIAL UNIQUE NOT NULL,
name VARCHAR(64) PRIMARY KEY,
description VARCHAR NOT NULL DEFAULT ''
);

CREATE TABLE stat_xp_cost (
stat VARCHAR(64) PRIMARY KEY,
xp_cost INT CHECK (xp_cost >= 0) NOT NULL,

FOREIGN KEY (stat) REFERENCES stat
);

CREATE TABLE influence_xp_cost (
influence VARCHAR(64) PRIMARY KEY,
xp_cost INT CHECK (xp_cost >= 0) NOT NULL,

FOREIGN KEY (influence) REFERENCES influence
);

CREATE TABLE humanity_xp_cost (
change_type VARCHAR(64) PRIMARY KEY,
xp_cost INT CHECK (xp_cost >= 0) NOT NULL
);

CREATE TABLE power_xp_cost (
in_clan BOOL NOT NULL,
level INT CHECK (level > 0) NOT NULL,
xp_cost INT CHECK (xp_cost >= 0) NOT NULL,

PRIMARY KEY (in_clan, level)
);

CREATE TABLE clan (
clan_id BIGSERIAL PRIMARY KEY NOT NULL,
name VARCHAR(64) UNIQUE NOT NULL,
unique_power VARCHAR(64) UNIQUE NOT NULL,
power_one VARCHAR(64) NOT NULL,
power_two VARCHAR(64) NOT NULL,
description VARCHAR NOT NULL DEFAULT '',

FOREIGN KEY (unique_power) REFERENCES power,
FOREIGN KEY (power_one) REFERENCES power,
FOREIGN KEY (power_two) REFERENCES power
);

CREATE TABLE character_status (
  name VARCHAR(16) PRIMARY KEY
);
INSERT INTO character_status (name) VALUES ('draft'), ('active'), ('inactive');

COMMIT;
