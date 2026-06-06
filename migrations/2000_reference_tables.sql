BEGIN;

CREATE TABLE stat (
stat_id BIGSERIAL UNIQUE NOT NULL,
id VARCHAR(64) PRIMARY KEY CHECK (id ~ '^[a-z-]+$'),
name VARCHAR(64) NOT NULL
);

CREATE TABLE power (
power_id BIGSERIAL UNIQUE NOT NULL,
id VARCHAR(64) PRIMARY KEY CHECK (id ~ '^[a-z-]+$'),
name VARCHAR(64) NOT NULL
);

CREATE TABLE influence (
influence_id BIGSERIAL UNIQUE NOT NULL,
id VARCHAR(64) PRIMARY KEY CHECK (id ~ '^[a-z-]+$'),
name VARCHAR(64) NOT NULL
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

INSERT INTO humanity_xp_cost (change_type, xp_cost) VALUES
  ('gain', 7),
  ('loss', 0)
;

CREATE TABLE power_xp_cost (
in_clan BOOL NOT NULL,
level INT CHECK (level > 0) NOT NULL,
xp_cost INT CHECK (xp_cost >= 0) NOT NULL,

PRIMARY KEY (in_clan, level)
);

INSERT INTO power_xp_cost (in_clan, level, xp_cost) VALUES
  (TRUE, 1, 9),
  (TRUE, 2, 12),
  (TRUE, 3, 15),
  (TRUE, 4, 18),
  (TRUE, 5, 21),
  (FALSE, 1, 12),
  (FALSE, 2, 15),
  (FALSE, 3, 18),
  (FALSE, 4, 21),
  (FALSE, 5, 24)
;

CREATE TABLE clan (
clan_id BIGSERIAL PRIMARY KEY NOT NULL,
name VARCHAR(64) UNIQUE NOT NULL,
unique_power VARCHAR(64) UNIQUE NOT NULL,
power_one VARCHAR(64) NOT NULL,
power_two VARCHAR(64) NOT NULL,

FOREIGN KEY (unique_power) REFERENCES power,
FOREIGN KEY (power_one) REFERENCES power,
FOREIGN KEY (power_two) REFERENCES power
);

CREATE TABLE character_status (
  name VARCHAR(16) PRIMARY KEY
);
INSERT INTO character_status (name) VALUES ('draft'), ('active'), ('inactive');

INSERT INTO stat (id, name) VALUES
  ('hp',                    'HP'),
  ('physical-ability',      'Fysisk Förmåga'),
  ('mental-ability',        'Mental Förmåga'),
  ('organizational-ability', 'Organisatorisk Förmåga')
;

INSERT INTO stat_xp_cost (stat, xp_cost) VALUES
  ('hp', 3),
  ('physical-ability', 4),
  ('mental-ability', 4),
  ('organizational-ability', 4)
;

INSERT INTO power (id, name) VALUES
  ('animalism', 'Animalism'),
  ('auspex',    'Auspex'),
  ('celerity',  'Celerity'),
  ('dominate',  'Dominate'),
  ('fortitude', 'Fortitude'),
  ('nightmare', 'Nightmare'),
  ('obfuscate', 'Obfuscate'),
  ('presence',  'Presence'),
  ('potence',   'Potence'),
  ('protean',   'Protean')
;

INSERT INTO influence (id, name) VALUES
  ('law',                'Juridik'),
  ('street-life',        'Gatuliv'),
  ('culture',            'Kultur'),
  ('police',             'Polis'),
  ('criminal',           'Kriminell'),
  ('university',         'Universitet'),
  ('high-society',       'Societet'),
  ('healthcare',         'Sjukvård'),
  ('politics',           'Politik'),
  ('occult',             'Ockult'),
  ('faith',              'Tro'),
  ('on-call',            'Jour'),
  ('transport',          'Transport'),
  ('technology',         'Teknik'),
  ('industry',           'Industri'),
  ('finance',            'Finans')
;

INSERT INTO influence_xp_cost (influence, xp_cost) VALUES
  ('law', 4),
  ('street-life', 4),
  ('culture', 4),
  ('police', 4),
  ('criminal', 4),
  ('university', 4),
  ('high-society', 4),
  ('healthcare', 4),
  ('politics', 4),
  ('occult', 4),
  ('faith', 4),
  ('on-call', 4),
  ('transport', 4),
  ('technology', 4),
  ('industry', 4),
  ('finance', 4)
;

CREATE TABLE covenant (
  covenant_id BIGSERIAL UNIQUE NOT NULL,
  id VARCHAR(64) PRIMARY KEY CHECK (id ~ '^[a-z-]+$'),
  name VARCHAR(64) UNIQUE NOT NULL
);

INSERT INTO covenant (id, name) VALUES
  ('carthian-movement',   'Karthiska Rörelsen'),
  ('circle-of-the-crone', 'Haggans Krans'),
  ('invictus',            'Invictus'),
  ('lancea-et-sanctum',   'Lancea et Sanctum')
;

INSERT INTO clan (name, unique_power, power_one, power_two) VALUES
  ('Ventrue',   'dominate',  'animalism', 'fortitude'),
  ('Nosferatu', 'nightmare', 'obfuscate', 'potence'),
  ('Daeva',     'presence',  'celerity',  'potence'),
  ('Gangrel',   'protean',   'animalism', 'fortitude'),
  ('Mekhet',    'auspex',    'celerity',  'obfuscate')
;

COMMIT;
