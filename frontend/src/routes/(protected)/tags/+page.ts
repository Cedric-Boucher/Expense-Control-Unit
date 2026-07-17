import { getTags } from '$lib/api';
import type { PageLoad } from '../../$types';

export const load: PageLoad = async () => {
	const tags = await getTags();
	return { tags };
};
