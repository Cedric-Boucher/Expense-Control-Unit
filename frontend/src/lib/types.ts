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
	parent_id: number | null;
	is_asset: boolean;
	created_at: string;
};

export type NewCategory = {
	name: string;
	parent_id?: number | null;
	is_asset?: boolean;
};

export type CategoryNode = Category & {
	children: CategoryNode[];
};

export type ImportCategory = {
	path: string[];
	created_at: string;
	is_asset: boolean;
};

export type ImportTransaction = {
	category_path: string[];
	amount: number;
	description: string;
	created_at: string;
};

export type ImportPayload = {
	categories: ImportCategory[];
	transactions: ImportTransaction[];
};
