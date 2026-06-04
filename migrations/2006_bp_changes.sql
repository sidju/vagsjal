BEGIN;

CREATE TABLE bp_change (
	bp_change_id BIGSERIAL PRIMARY KEY NOT NULL,
	vampire_id BIGINT NOT NULL,
	change INT NOT NULL CHECK (change != 0),
	note VARCHAR(256) NOT NULL DEFAULT '',
	creation_time TIMESTAMPTZ DEFAULT NOW() NOT NULL,

	FOREIGN KEY (vampire_id) REFERENCES vampire ON DELETE CASCADE
);

CREATE OR REPLACE VIEW vampire_stat AS
	WITH vampire_bp AS (
		SELECT
			v.vampire_id,
			(FLOOR(
				EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - v.date_embraced::TIMESTAMPTZ - v.torpor_time))
				/ (86400.0 * 365.25 * 24)
			) + 1 + COALESCE(SUM(bpc.change), 0))::INT AS bp
		FROM vampire v
		LEFT JOIN bp_change bpc ON bpc.vampire_id = v.vampire_id
		GROUP BY v.vampire_id, v.date_embraced, v.torpor_time
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
		SELECT v.vampire_id, s.name,
			6 + COALESCE(SUM(sr.increase), 0)
				+ CASE WHEN s.name = 'HP' THEN COALESCE(vb.bp, 0) * 6 ELSE 0 END AS value,
			BOOL_OR(sr.stat_raise_id IS NOT NULL AND srr.stat_raise_id IS NULL) AS pending_review
		FROM vampire v
		JOIN stat s ON TRUE
		LEFT JOIN vampire_bp vb ON vb.vampire_id = v.vampire_id
		LEFT JOIN stat_raise sr ON sr.vampire_id = v.vampire_id AND sr.stat = s.name
		LEFT JOIN stat_raise_review srr ON srr.stat_raise_id = sr.stat_raise_id
			AND (srr.state IS NULL OR srr.state != 'denied')
		GROUP BY v.vampire_id, s.name, vb.bp
	)
;

COMMIT;
