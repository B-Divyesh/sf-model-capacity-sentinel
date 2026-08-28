-- Keep the legacy column only long enough for the application startup
-- migration to encrypt existing canaries and blank the plaintext values.
ALTER TABLE probes ADD COLUMN prompt_cipher TEXT NOT NULL DEFAULT '';
