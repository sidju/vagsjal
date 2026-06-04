BEGIN;

CREATE TABLE stat (
stat_id BIGSERIAL UNIQUE NOT NULL,
id VARCHAR(64) PRIMARY KEY CHECK (id ~ '^[a-z-]+$'),
name VARCHAR(64) NOT NULL,
description VARCHAR NOT NULL DEFAULT ''
);

CREATE TABLE power (
power_id BIGSERIAL UNIQUE NOT NULL,
id VARCHAR(64) PRIMARY KEY CHECK (id ~ '^[a-z-]+$'),
name VARCHAR(64) NOT NULL,
description VARCHAR NOT NULL DEFAULT ''
);

CREATE TABLE influence (
influence_id BIGSERIAL UNIQUE NOT NULL,
id VARCHAR(64) PRIMARY KEY CHECK (id ~ '^[a-z-]+$'),
name VARCHAR(64) NOT NULL,
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

INSERT INTO stat (id, name, description) VALUES
  ('hp',                    'HP',                     'How much damage you can take before falling in combat. Base value is Blood Potency * 6 + 6.'),
  ('physical-ability',      'Fysisk Förmåga',         'Your ability to overcome physical challenges and fight.'),
  ('mental-ability',        'Mental Förmåga',          'Your ability to resist mental manipulation and inflict it on others.'),
  ('organizational-ability', 'Organisatorisk Förmåga', 'Your ability to keep track of and maintain investments.')
;

INSERT INTO stat_xp_cost (stat, xp_cost) VALUES
  ('hp', 3),
  ('physical-ability', 4),
  ('mental-ability', 4),
  ('organizational-ability', 4)
;

INSERT INTO power (id, name, description) VALUES
  ('animalism', 'Animalism', 'Powers to commune with, command, and transform through beasts.'),
  ('auspex',    'Auspex',    'Powers to sense emotions, pierce illusions, and read minds.'),
  ('celerity',  'Celerity',  'Powers to act with supernatural speed.'),
  ('dominate',  'Dominate',  'Powers to control minds and memories.'),
  ('fortitude', 'Fortitude', 'Powers to resist damage and mental attacks.'),
  ('nightmare', 'Nightmare', 'Powers to inspire fear and project terrifying illusions.'),
  ('obfuscate', 'Obfuscate', 'Powers to hide, disguise, and distort perception.'),
  ('presence',  'Presence',  'Powers to influence emotions and social standing.'),
  ('potence',   'Potence',   'Powers to strike harder and perform inhuman feats of strength.'),
  ('protean',   'Protean',   'Powers to reshape the body into predatory and monstrous forms.')
;

INSERT INTO influence (id, name, description) VALUES
  ('law',                'Juridik',    'Lagens byråkratier och institutioner.'),
  ('street-life',        'Gatuliv',    'Pubarna, klubbarna och deras gäster som håller natten vid liv.'),
  ('culture',            'Kultur',     'Allt från museer och konserthus till illegal gatukonst.'),
  ('police',             'Polis',      'Från konstapeln på hörnet, till tjänstemannen som sköter pappersarbetet, till polischefen.'),
  ('criminal',           'Kriminell',  'Varje kategori av konsekvent kriminell verksamhet, från skattebedrägeri till gängvåld.'),
  ('university',         'Universitet','Institutionerna själva, studenterna och offentliga bibliotek och arkiv.'),
  ('high-society',       'Societet',   'De rika och adliga i samhället, samt alla kontakter för att tillgodose deras behov.'),
  ('healthcare',         'Sjukvård',   'Sjukhus, kliniker, apotek och deras personal.'),
  ('politics',           'Politik',    'Myndighetsanställda och politiker, användbara för bygglov, miljökvoter osv.'),
  ('occult',             'Ockult',     'Hemliga sällskap, ockulta utövare och samlingar av mystisk kunskap.'),
  ('faith',              'Tro',        'Varje organiserad religiös struktur, och till och med vissa grupper av de oorganiserade.'),
  ('on-call',            'Jour',       'Brandmän och alla slags jourtjänster (säkerhet, skadedjursbekämpning, inspektioner, underhåll).'),
  ('transport',          'Transport',  'Nätverken och människorna som flyttar saker och människor.'),
  ('technology',         'Teknik',     'IT, kemi osv. De avancerade delarna av samhället och experterna som sköter dem.'),
  ('industry',           'Industri',   'Både byggföretag och industrier tillsammans med deras arbetare.'),
  ('finance',            'Finans',     'Både banker och aktiemarknaden, den enda källan till detaljerad finansiell data.')
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

INSERT INTO clan (name, unique_power, power_one, power_two, description) VALUES
  ('Ventrue',   'dominate',  'animalism', 'fortitude', 'Vampires of command, endurance, and careful control.'),
  ('Nosferatu', 'nightmare', 'obfuscate', 'potence',   'Vampires who weaponize fear, concealment, and brute force.'),
  ('Daeva',     'presence',  'celerity',  'potence',   'Vampires of allure, speed, and overwhelming force.'),
  ('Gangrel',   'protean',   'animalism', 'fortitude', 'Vampires who hunt through beast, flesh, and resilience.'),
  ('Mekhet',    'auspex',    'celerity',  'obfuscate', 'Vampires of insight, speed, and shadowed perception.')
;

COMMIT;
