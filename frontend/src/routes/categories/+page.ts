import { getCategories } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
    const categories = await getCategories();
    return { categories };
};
