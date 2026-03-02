CREATE TABLE category_hierarchy (
    category_id INTEGER PRIMARY KEY,
    parent_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,

    CONSTRAINT fk_child_user 
        FOREIGN KEY (category_id, user_id) 
        REFERENCES categories (id, user_id) ON DELETE CASCADE,

    CONSTRAINT fk_parent_user 
        FOREIGN KEY (parent_id, user_id) 
        REFERENCES categories (id, user_id) ON DELETE RESTRICT,

    CONSTRAINT no_self_parenting CHECK (category_id <> parent_id)
);
