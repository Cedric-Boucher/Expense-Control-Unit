export type Transaction = {
    id: number;
    category: Category;
    description: string;
    amount: number;
    created_at: string;
};

export type NewTransaction = {
    description: string;
    category_id: number;
    amount: number;
    created_at?: string;
};

export type NewUser = {
    username: string;
    password: string;
};

export type User = {
    id: string;
    username: string;
    password_hash: string;
    created_at: string;
};

export type Category = {
    id: number;
    name: string;
    created_at: string;
}

export type NewCategory = {
    name: string;
}
