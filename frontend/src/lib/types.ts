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
