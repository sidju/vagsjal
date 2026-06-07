BEGIN;

CREATE VIEW xp_remaining AS
	SELECT vampire_id, SUM(amount) AS amount
	FROM ((
		SELECT vampire_id, amount
		FROM xp
	) UNION ALL (
		SELECT humanity_change.vampire_id, (-xp_cost) AS amount
		FROM humanity_change
		LEFT JOIN humanity_change_review USING (humanity_change_id)
		WHERE humanity_change_review.state IS NULL OR humanity_change_review.state != 'denied'
	) UNION ALL (
		SELECT influence_raise.vampire_id, (-xp_cost) AS amount
		FROM influence_raise
		LEFT JOIN influence_raise_review USING (influence_raise_id)
		WHERE influence_raise_review.state IS NULL OR influence_raise_review.state != 'denied'
	) UNION ALL (
		SELECT power_raise.vampire_id, (-xp_cost) AS amount
		FROM power_raise
		LEFT JOIN power_raise_review USING (power_raise_id)
		WHERE power_raise_review.state IS NULL OR power_raise_review.state != 'denied'
	) UNION ALL (
		SELECT stat_raise.vampire_id, (-xp_cost) AS amount
		FROM stat_raise
		LEFT JOIN stat_raise_review USING (stat_raise_id)
		WHERE stat_raise_review.state IS NULL OR stat_raise_review.state != 'denied'
	)) GROUP BY vampire_id
;

CREATE VIEW vampire_stat AS
	WITH vampire_bp AS (
		SELECT
			vampire_id,
			(FLOOR(
				EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - date_embraced::TIMESTAMPTZ - torpor_time))
				/ (86400.0 * 365.25 * 24)
			) + 1)::INT AS bp
		FROM vampire
	)
	SELECT vampire_id, id AS "id!", name AS "name!", COALESCE(value, 0) AS "value!", COALESCE(pending_review, false) AS "pending_review!"
	FROM (
		SELECT v.vampire_id, 'humanity' AS id, 'Mänsklighet' AS name,
			7 + COALESCE(SUM(CASE WHEN hcr.state IS NULL OR hcr.state != 'denied' THEN hc.change ELSE 0 END), 0) AS value,
			BOOL_OR(hc.humanity_change_id IS NOT NULL AND hcr.humanity_change_id IS NULL) AS pending_review
		FROM vampire v
		LEFT JOIN humanity_change hc ON hc.vampire_id = v.vampire_id
		LEFT JOIN humanity_change_review hcr ON hcr.humanity_change_id = hc.humanity_change_id
		GROUP BY v.vampire_id
	) UNION (
		SELECT vampire_id, 'blood-potency' AS id, 'Blodpotens' AS name, bp AS value, false AS pending_review
		FROM vampire_bp
	) UNION (
		SELECT v.vampire_id, s.id, s.name,
			6 + COALESCE(SUM(CASE WHEN srr.state IS NULL OR srr.state != 'denied' THEN sr.increase ELSE 0 END), 0)
				+ CASE
				WHEN s.id = 'hp' THEN COALESCE(vb.bp, 0) * 6
				WHEN s.id = 'organizational-ability' THEN -(COALESCE(vb.bp, 0) * 2)
				ELSE 0
			END AS value,
			BOOL_OR(sr.stat_raise_id IS NOT NULL AND srr.stat_raise_id IS NULL) AS pending_review
		FROM vampire v
		JOIN stat s ON TRUE
		LEFT JOIN vampire_bp vb ON vb.vampire_id = v.vampire_id
		LEFT JOIN stat_raise sr ON sr.vampire_id = v.vampire_id AND sr.stat = s.id
		LEFT JOIN stat_raise_review srr ON srr.stat_raise_id = sr.stat_raise_id
		GROUP BY v.vampire_id, s.id, s.name, vb.bp
	)
;

CREATE VIEW vampire_power AS
	SELECT
		power_raise.vampire_id,
		power.id AS "id!",
		power.name AS "name!",
		COALESCE(SUM(power_raise.increase), 0)::INT AS "value!",
		BOOL_OR(power_raise_review.power_raise_id IS NULL) AS "pending_review!"
	FROM power_raise
	LEFT JOIN power_raise_review USING (power_raise_id)
	JOIN power ON power.id = power_raise.power
	WHERE power_raise_review.state IS NULL OR power_raise_review.state != 'denied'
	GROUP BY power_raise.vampire_id, power.id, power.name
	ORDER BY SUM(power_raise.increase) DESC
;

CREATE VIEW vampire_influence AS
	SELECT
		influence_raise.vampire_id,
		influence.id AS "id!",
		influence.name AS "name!",
		COALESCE(SUM(influence_raise.increase), 0)::INT AS "value!",
		BOOL_OR(influence_raise_review.influence_raise_id IS NULL) AS "pending_review!"
	FROM influence_raise
	LEFT JOIN influence_raise_review USING (influence_raise_id)
	JOIN influence ON influence.id = influence_raise.influence
	WHERE influence_raise_review.state IS NULL OR influence_raise_review.state != 'denied'
	GROUP BY influence_raise.vampire_id, influence.id, influence.name
	ORDER BY SUM(influence_raise.increase) DESC
;

COMMIT;
