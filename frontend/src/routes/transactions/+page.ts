import { getTransactions } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
    const transactions = await getTransactions();
    return { transactions };
};
