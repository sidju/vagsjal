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
	SELECT vampire_id, name AS "name!", COALESCE(value, 0) AS "value!", COALESCE(pending_review, false) AS "pending_review!"
	FROM (
		SELECT v.vampire_id, 'Humanity' AS name, 7 + COALESCE(SUM(hc.change), 0) AS value, BOOL_OR(hc.humanity_change_id IS NOT NULL AND hcr.humanity_change_id IS NULL) AS pending_review
		FROM vampire v
		LEFT JOIN humanity_change hc ON hc.vampire_id = v.vampire_id
		LEFT JOIN humanity_change_review hcr ON hcr.humanity_change_id = hc.humanity_change_id AND (hcr.state IS NULL OR hcr.state != 'denied')
		GROUP BY v.vampire_id
	) UNION (
		SELECT vampire_id, 'Blood Potency' AS name, bp AS value, false AS pending_review
		FROM vampire_bp
	) UNION (
		-- HP: BP*6+6 base plus any purchased raises
		SELECT
			vampire_bp.vampire_id,
			'HP' AS name,
			(vampire_bp.bp * 6 + 6 + COALESCE(SUM(stat_raise.increase), 0)) AS value,
			BOOL_OR(stat_raise.stat_raise_id IS NOT NULL AND stat_raise_review.stat_raise_id IS NULL) AS pending_review
		FROM vampire_bp
		LEFT JOIN stat_raise ON stat_raise.vampire_id = vampire_bp.vampire_id AND stat_raise.stat = 'HP'
		LEFT JOIN stat_raise_review USING (stat_raise_id)
		WHERE stat_raise_review.state IS NULL OR stat_raise_review.state != 'denied'
		GROUP BY vampire_bp.vampire_id, vampire_bp.bp
	) UNION (
		-- All non-HP stats (PA, MA, OA, ...)
		SELECT stat_raise.vampire_id, stat AS name, SUM(increase) AS value, BOOL_OR(stat_raise_review.stat_raise_id IS NULL) AS pending_review
		FROM stat_raise
		LEFT JOIN stat_raise_review USING (stat_raise_id)
		WHERE (stat_raise_review.state IS NULL OR stat_raise_review.state != 'denied') AND stat != 'HP'
		GROUP BY stat, stat_raise.vampire_id
	)
;

CREATE VIEW vampire_power AS
	SELECT
		vampire_id,
		power AS "name!",
		COALESCE(SUM(increase), 0) AS "value!",
		BOOL_OR(power_raise_review.power_raise_id IS NULL) AS "pending_review!"
	FROM power_raise
	LEFT JOIN power_raise_review USING (power_raise_id)
	WHERE power_raise_review.state IS NULL OR power_raise_review.state != 'denied'
	GROUP BY power, vampire_id
	ORDER BY SUM(increase) DESC
;

CREATE VIEW vampire_influence AS
	SELECT
		vampire_id,
		influence AS "name!",
		COALESCE(SUM(increase), 0) AS "value!",
		BOOL_OR(influence_raise_review.influence_raise_id IS NULL) AS "pending_review!"
	FROM influence_raise
	LEFT JOIN influence_raise_review USING (influence_raise_id)
	WHERE influence_raise_review.state IS NULL OR influence_raise_review.state != 'denied'
	GROUP BY influence, vampire_id
	ORDER BY SUM(increase) DESC
;

COMMIT;
