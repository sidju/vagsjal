BEGIN;

INSERT INTO stat (name, description) VALUES
  ('HP',                     'How much damage you can take before falling in combat. Base value is Blood Potency * 6 + 6.'),
  ('Physical Ability',       'Your ability to overcome physical challenges and fight.'),
  ('Mental Ability',         'Your ability to resist mental manipulation and inflict it on others.'),
  ('Organizational Ability', 'Your ability to keep track of and maintain investments.')
;

INSERT INTO power (name, description) VALUES
  ('Animalism', 'Powers to manipulate beasts, both mundane and kindred.'),
  ('Auspex',    'Powers to see and show truth.'),
  ('Dominate',  'Powers to control minds and even memories.'),
  ('Presence',  'Powers to control emotions.'),
  ('Obfuscate', 'Powers to control the perception of others.'),
  ('Nightmare', 'Powers to show people what they fear.'),
  ('Protean',   'Powers to change shape.')
;

INSERT INTO influence (name) VALUES
  ('Juridik'),
  ('Gatuliv'),
  ('Societet'),
  ('Kultur')
;

INSERT INTO clan (name, unique_power, power_one, power_two) VALUES
  ('Ventrue',   'Dominate',  'Animalism', 'Presence'),
  ('Nosferatu', 'Obfuscate', 'Animalism', 'Nightmare')
;

INSERT INTO app_user (user_id, email, role) VALUES
  (0, 'dummy',                   'user'),
  (1, 'storyteller@example.com', 'storyteller'),
  (2, 'player@example.com',      'user')
;

INSERT INTO vampire (vampire_id, user_id, active, name, apparent_age, date_embraced, torpor_time, clan_id) VALUES
  (1, 2, TRUE, 'John Smith', 32, '1999-01-08', '0 years', 1),
  (2, 2, TRUE, 'Jane Doe',   28, '1985-06-15', '5 years', 2)
;

INSERT INTO xp (vampire_id, amount) VALUES
  (1, 48),
  (2, 36)
;


-- Humanity (character creation baseline, both approved; third pending)
INSERT INTO humanity_change (humanity_change_id, vampire_id, change, xp_cost, note) VALUES
  (1, 1,  7, 0, 'Character creation'),
  (2, 2,  6, 0, 'Character creation'),
  (3, 1, -1, 0, 'Witnessed a massacre')
;
INSERT INTO humanity_change_review (humanity_change_id, state, reviewer_id) VALUES
  (1, 'approved', 1),
  (2, 'approved', 1)
;

-- Stat raises
INSERT INTO stat_raise (stat_raise_id, vampire_id, stat, increase, xp_cost) VALUES
  (1, 1, 'Physical Ability', 3, 12), -- approved
  (2, 1, 'Mental Ability',   2,  8), -- denied
  (3, 2, 'HP',               6, 18)  -- pending
;
INSERT INTO stat_raise_review (stat_raise_id, state, reviewer_id) VALUES
  (1, 'approved', 1),
  (2, 'denied',   1)
;

-- Power raises
INSERT INTO power_raise (power_raise_id, vampire_id, power, increase, xp_cost) VALUES
  (1, 1, 'Dominate',  1,  9), -- approved (clan power)
  (2, 1, 'Presence',  1,  9), -- approved (clan power)
  (3, 2, 'Obfuscate', 2, 21), -- approved (clan power lvl1+lvl2)
  (4, 2, 'Nightmare', 1, 12)  -- pending
;
INSERT INTO power_raise_review (power_raise_id, state, reviewer_id) VALUES
  (1, 'approved', 1),
  (2, 'approved', 1),
  (3, 'approved', 1)
;

-- Influence raises
INSERT INTO influence_raise (influence_raise_id, vampire_id, influence, increase, xp_cost) VALUES
  (1, 1, 'Juridik',  2, 8), -- approved
  (2, 2, 'Gatuliv',  1, 4), -- denied
  (3, 1, 'Societet', 1, 4)  -- pending
;
INSERT INTO influence_raise_review (influence_raise_id, state, reviewer_id) VALUES
  (1, 'approved', 1),
  (2, 'denied',   1)
;

COMMIT;
