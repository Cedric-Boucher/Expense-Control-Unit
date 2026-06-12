ALTER TABLE categories 
ADD COLUMN parent_id INTEGER REFERENCES categories(id) ON DELETE RESTRICT;

ALTER TABLE categories 
ADD CONSTRAINT no_self_parenting CHECK (id <> parent_id);

UPDATE categories c 
SET parent_id = ch.parent_id 
FROM category_hierarchy ch 
WHERE c.id = ch.category_id;

DROP TABLE category_hierarchy;

ALTER TABLE categories 
DROP CONSTRAINT IF EXISTS categories_user_id_name_key;

CREATE UNIQUE INDEX categories_unique_name_per_parent 
ON categories (user_id, name, COALESCE(parent_id, -1));

CREATE OR REPLACE FUNCTION check_category_cycle()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.parent_id IS NULL THEN
        RETURN NEW;
    END IF;

    IF EXISTS (
        WITH RECURSIVE subordinates AS (
            SELECT id FROM categories WHERE parent_id = NEW.id
            UNION ALL
            SELECT c.id 
            FROM categories c
            INNER JOIN subordinates s ON c.parent_id = s.id
        )
        SELECT 1 FROM subordinates WHERE id = NEW.parent_id
    ) THEN
        RAISE EXCEPTION 'Circular dependency detected: Category % cannot be a child of its own descendant %', NEW.id, NEW.parent_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_prevent_category_cycles
BEFORE UPDATE ON categories
FOR EACH ROW 
WHEN (NEW.parent_id IS DISTINCT FROM OLD.parent_id)
EXECUTE FUNCTION check_category_cycle();
