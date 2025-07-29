export const ssr = false;
export const prerender = true;

import { load_user } from "$lib/api";
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = load_user;
