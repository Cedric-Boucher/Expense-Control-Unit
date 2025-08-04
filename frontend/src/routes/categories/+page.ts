import { getCategories, load_user } from '$lib/api';
import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
    const { user } = await load_user();

    if (!user) {
        throw redirect(302, '/login');
    }
    
    const categories = await getCategories();
    return { categories };
};
