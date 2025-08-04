<script lang="ts">
    import type { Category } from '$lib/types';
    import { formatTimestampLocalForDisplay } from '$lib/utils';
    import { goto } from '$app/navigation';
	import { page } from '$app/state';

    export let category: Category;
    export let showActions: boolean = true;

    // Optional overrides
    export let onEdit: (() => void) | null = null;
    export let onDelete: (() => void) | null = null;

    function handleEdit() {
        if (onEdit) {
            onEdit();
        } else {
            goto(`/categories/${category.id}/edit?redirectTo=${encodeURIComponent(page.url.pathname)}`);
        }
    }

    function handleDelete() {
        if (onDelete) {
            onDelete();
        } else {
            goto(`/categories/${category.id}/delete?redirectTo=${encodeURIComponent(page.url.pathname)}`);
        }
    }
</script>

<li class="bg-white dark:bg-gray-800 shadow rounded p-4 flex justify-between items-start gap-4">
    <!-- Left: category details -->
    <div>
        <div class="text-2xl font-bold">{category.name}</div>
        <div class="text-gray-500 text-sm">Created: {formatTimestampLocalForDisplay(category.created_at)}</div>
    </div>

    <!-- Right: actions -->
    {#if showActions}
        <div class="flex flex-col gap-2">
            <button
                type="button"
                class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
                on:click={handleEdit}
            >
                Edit
            </button>
            <button
                type="button"
                class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
                on:click={handleDelete}
            >
                Delete
            </button>
        </div>
    {/if}
</li>
