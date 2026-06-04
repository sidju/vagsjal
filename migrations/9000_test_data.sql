BEGIN;

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

INSERT INTO app_user (user_id, name, oidc_subject, role) VALUES
  (0, 'Storyteller', 'storyteller-subject', 'storyteller'),
  (1, 'Reviewer',    'reviewer-subject',    'storyteller'),
  (2, 'Player',      'player-subject',      'user')
;

INSERT INTO vampire (vampire_id, user_id, status, name, apparent_age, date_embraced, torpor_time, clan_id) VALUES
  (1, 2, 'active', 'John Smith', 32, '1999-01-08', '0 years', 1),
  (2, 2, 'active', 'Jane Doe',   28, '1985-06-15', '5 years', 2)
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
  (1, 1, 'physical-ability', 3, 12), -- approved
  (2, 1, 'mental-ability',   2,  8), -- denied
  (3, 2, 'hp',               6, 18)  -- pending
;
INSERT INTO stat_raise_review (stat_raise_id, state, reviewer_id) VALUES
  (1, 'approved', 1),
  (2, 'denied',   1)
;

-- Power raises (each row is always +1)
INSERT INTO power_raise (power_raise_id, vampire_id, power, xp_cost) VALUES
  (1, 1, 'dominate',  9), -- approved (clan power)
  (2, 1, 'presence',  9), -- approved (clan power)
  (3, 2, 'obfuscate', 9), -- approved (clan power, lvl1)
  (4, 2, 'obfuscate', 12), -- approved (clan power, lvl2)
  (5, 2, 'nightmare', 12)  -- pending
;
INSERT INTO power_raise_review (power_raise_id, state, reviewer_id) VALUES
  (1, 'approved', 1),
  (2, 'approved', 1),
  (3, 'approved', 1),
  (4, 'approved', 1)
;

-- Influence raises (each row is always +1)
INSERT INTO influence_raise (influence_raise_id, vampire_id, influence, xp_cost) VALUES
  (1, 1, 'law',  4), -- approved
  (2, 1, 'law',  4), -- approved
  (3, 2, 'street-life',  4), -- denied
  (4, 1, 'high-society', 4)  -- pending
;
INSERT INTO influence_raise_review (influence_raise_id, state, reviewer_id) VALUES
  (1, 'approved', 1),
  (2, 'denied',   1)
;

-- Reset sequences after explicit ID inserts so BIGSERIAL continues from the right value.
SELECT setval('app_user_user_id_seq',                 (SELECT MAX(user_id)           FROM app_user));
SELECT setval('vampire_vampire_id_seq',               (SELECT MAX(vampire_id)        FROM vampire));
SELECT setval('humanity_change_humanity_change_id_seq', (SELECT MAX(humanity_change_id) FROM humanity_change));
SELECT setval('stat_raise_stat_raise_id_seq',           (SELECT MAX(stat_raise_id)       FROM stat_raise));
SELECT setval('power_raise_power_raise_id_seq',         (SELECT MAX(power_raise_id)       FROM power_raise));
SELECT setval('influence_raise_influence_raise_id_seq', (SELECT MAX(influence_raise_id)   FROM influence_raise));

COMMIT;
