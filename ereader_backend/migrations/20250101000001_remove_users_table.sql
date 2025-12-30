-- Remove foreign key constraints from all tables that reference users
ALTER TABLE devices DROP CONSTRAINT IF EXISTS devices_user_id_fkey;
ALTER TABLE books DROP CONSTRAINT IF EXISTS books_user_id_fkey;
ALTER TABLE collections DROP CONSTRAINT IF EXISTS collections_user_id_fkey;
ALTER TABLE reading_states DROP CONSTRAINT IF EXISTS reading_states_user_id_fkey;
ALTER TABLE annotations DROP CONSTRAINT IF EXISTS annotations_user_id_fkey;

-- Drop the users table
DROP TABLE IF EXISTS users CASCADE;
