BEGIN;

INSERT INTO stat (name, description) VALUES
  ('HP',                     'How much damage you can take before falling in combat. Base value is Blood Potency * 6 + 6.'),
  ('Physical Ability',       'Your ability to overcome physical challenges and fight.'),
  ('Mental Ability',         'Your ability to resist mental manipulation and inflict it on others.'),
  ('Organizational Ability', 'Your ability to keep track of and maintain investments.')
;

INSERT INTO power (name, description) VALUES
  ('Animalism', 'Powers to commune with, command, and transform through beasts.'),
  ('Auspex',    'Powers to sense emotions, pierce illusions, and read minds.'),
  ('Celerity',  'Powers to act with supernatural speed.'),
  ('Dominate',  'Powers to control minds and memories.'),
  ('Fortitude', 'Powers to resist damage and mental attacks.'),
  ('Nightmare', 'Powers to inspire fear and project terrifying illusions.'),
  ('Obfuscate', 'Powers to hide, disguise, and distort perception.'),
  ('Presence',  'Powers to influence emotions and social standing.'),
  ('Potence',   'Powers to strike harder and perform inhuman feats of strength.'),
  ('Protean',   'Powers to reshape the body into predatory and monstrous forms.')
;

INSERT INTO clan (name, unique_power, power_one, power_two, description) VALUES
  ('Ventrue',   'Dominate',  'Animalism', 'Fortitude', 'Vampires of command, endurance, and careful control.'),
  ('Nosferatu', 'Nightmare', 'Obfuscate', 'Potence',   'Vampires who weaponize fear, concealment, and brute force.'),
  ('Daeva',     'Presence',  'Celerity',  'Potence',   'Vampires of allure, speed, and overwhelming force.'),
  ('Gangrel',   'Protean',   'Animalism', 'Fortitude', 'Vampires who hunt through beast, flesh, and resilience.'),
  ('Mekhet',    'Auspex',    'Celerity',  'Obfuscate', 'Vampires of insight, speed, and shadowed perception.')
;

INSERT INTO influence (name, description) VALUES
  ('Juridik',    'Lagens byråkratier och institutioner.'),
  ('Gatuliv',    'Pubarna, klubbarna och deras gäster som håller natten vid liv.'),
  ('Kultur',     'Allt från museer och konserthus till illegal gatukonst.'),
  ('Polis',      'Från konstapeln på hörnet, till tjänstemannen som sköter pappersarbetet, till polischefen.'),
  ('Kriminell',  'Varje kategori av konsekvent kriminell verksamhet, från skattebedrägeri till gängvåld.'),
  ('Universitet','Institutionerna själva, studenterna och offentliga bibliotek och arkiv.'),
  ('Societet',   'De rika och adliga i samhället, samt alla kontakter för att tillgodose deras behov.'),
  ('Sjukvård',   'Sjukhus, kliniker, apotek och deras personal.'),
  ('Politik',    'Myndighetsanställda och politiker, användbara för bygglov, miljökvoter osv.'),
  ('Ockult',     'Hemliga sällskap, ockulta utövare och samlingar av mystisk kunskap.'),
  ('Tro',        'Varje organiserad religiös struktur, och till och med vissa grupper av de oorganiserade.'),
  ('Jour',       'Brandmän och alla slags jourtjänster (säkerhet, skadedjursbekämpning, inspektioner, underhåll).'),
  ('Transport',  'Nätverken och människorna som flyttar saker och människor.'),
  ('Teknik',     'IT, kemi osv. De avancerade delarna av samhället och experterna som sköter dem.'),
  ('Industri',   'Både byggföretag och industrier tillsammans med deras arbetare.'),
  ('Finans',     'Både banker och aktiemarknaden, den enda källan till detaljerad finansiell data.')
;

INSERT INTO influence_xp_cost (influence, xp_cost) VALUES
  ('Juridik', 4),
  ('Gatuliv', 4),
  ('Kultur', 4),
  ('Polis', 4),
  ('Kriminell', 4),
  ('Universitet', 4),
  ('Societet', 4),
  ('Sjukvård', 4),
  ('Politik', 4),
  ('Ockult', 4),
  ('Tro', 4),
  ('Jour', 4),
  ('Transport', 4),
  ('Teknik', 4),
  ('Industri', 4),
  ('Finans', 4)
;

INSERT INTO stat_xp_cost (stat, xp_cost) VALUES
  ('HP', 3),
  ('Physical Ability', 4),
  ('Mental Ability', 4),
  ('Organizational Ability', 4)
;

COMMIT;
