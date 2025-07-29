export type Transaction = {
    id: number;
    description: string;
    amount: number;
    created_at: string;
};

export type NewTransaction = {
    description: string;
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
