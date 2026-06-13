-- 1. Create composite UNIQUE constraints on the parent tables. 
-- (The database needs these to act as targets for our new composite foreign keys).
ALTER TABLE transactions ADD CONSTRAINT transactions_id_user_id_key UNIQUE (id, user_id);
ALTER TABLE tags ADD CONSTRAINT tags_id_user_id_key UNIQUE (id, user_id);

-- 2. Add the user_id column to the junction table (nullable at first so we can backfill)
ALTER TABLE transaction_tags ADD COLUMN user_id UUID;

-- 3. Backfill the existing data by pulling the user_id from the transactions table
UPDATE transaction_tags tt
SET user_id = t.user_id
FROM transactions t
WHERE tt.transaction_id = t.id;

-- 4. Enforce the NOT NULL constraint now that the data is populated
ALTER TABLE transaction_tags ALTER COLUMN user_id SET NOT NULL;

-- 5. Drop the old, simple foreign keys
ALTER TABLE transaction_tags DROP CONSTRAINT transaction_tags_transaction_id_fkey;
ALTER TABLE transaction_tags DROP CONSTRAINT transaction_tags_tag_id_fkey;

-- 6. Add the new strict Composite Foreign Keys
ALTER TABLE transaction_tags 
  ADD CONSTRAINT transaction_tags_txn_user_fkey 
  FOREIGN KEY (transaction_id, user_id) REFERENCES transactions(id, user_id) ON DELETE CASCADE;

ALTER TABLE transaction_tags 
  ADD CONSTRAINT transaction_tags_tag_user_fkey 
  FOREIGN KEY (tag_id, user_id) REFERENCES tags(id, user_id) ON DELETE CASCADE;
