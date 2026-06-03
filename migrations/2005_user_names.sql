BEGIN;

ALTER TABLE app_user
  ADD COLUMN name VARCHAR(256) NOT NULL DEFAULT '';

UPDATE app_user
SET name = oidc_subject
WHERE name = '';

COMMIT;
