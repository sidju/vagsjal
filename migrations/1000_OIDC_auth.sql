BEGIN; -- Work in a transaction

-- Required for proper OIDC login, and enables redirecting users back to where
-- they were before they were redirected to log in again.
CREATE TABLE login_process (
creation_time TIMESTAMPTZ DEFAULT NOW(), -- To clean old processes
state_id VARCHAR PRIMARY KEY, -- Randomly generated, collisions improbable
nonce VARCHAR NOT NULL -- Used to validate the OIDC response
);

-- To track our internal state (and registrations) we need to have user entries
CREATE TABLE app_role (
name VARCHAR(64) PRIMARY KEY
);
INSERT INTO app_role (name) VALUES ('user'), ('storyteller');

CREATE TABLE app_user (
user_id BIGSERIAL PRIMARY KEY,
email VARCHAR(256) NOT NULL UNIQUE,
role VARCHAR(64) NOT NULL DEFAULT 'user',

FOREIGN KEY (role) REFERENCES app_role
);

-- OIDC login takes care of authentication and user metadata, but we still need
-- to manage the sessions ourselves.
CREATE TABLE session (
session_id VARCHAR PRIMARY KEY, -- Randomly generated, collisions improbable
user_id INTEGER NOT NULL,
valid_until TIMESTAMPTZ DEFAULT NOW() + '6 hours',

FOREIGN KEY (user_id) REFERENCES app_user
);

COMMIT; -- Apply the transaction
