CREATE OR REPLACE FUNCTION check_category_cycle()
RETURNS TRIGGER AS $$
BEGIN
    -- If the new parent is already a descendant of the category, we have a loop
    IF EXISTS (
        WITH RECURSIVE subordinates AS (
            -- Start with the category being moved/inserted
            SELECT category_id FROM category_hierarchy WHERE parent_id = NEW.category_id
            UNION ALL
            -- Find all children of those children
            SELECT ch.category_id 
            FROM category_hierarchy ch
            INNER JOIN subordinates s ON ch.parent_id = s.category_id
        )
        SELECT 1 FROM subordinates WHERE category_id = NEW.parent_id
    ) THEN
        RAISE EXCEPTION 'Circular dependency detected: Category % cannot be a child of its own descendant %', NEW.category_id, NEW.parent_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_prevent_category_cycles
BEFORE INSERT OR UPDATE ON category_hierarchy
FOR EACH ROW EXECUTE FUNCTION check_category_cycle();
