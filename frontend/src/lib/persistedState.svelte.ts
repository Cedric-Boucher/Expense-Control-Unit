export function createPersistedState<T>(key: string, initialValue: T) {
	let state = $state<T>(initialValue);

	if (typeof window !== 'undefined') {
		const stored = localStorage.getItem(key);
		if (stored) {
			try {
				state = JSON.parse(stored);
			} catch (e) {
				console.error(`Failed to parse localStorage for ${key}`, e);
			}
		}

		$effect(() => {
			localStorage.setItem(key, JSON.stringify(state));
		});
	}

	return {
		get value() {
			return state;
		},
		set value(v: T) {
			state = v;
		}
	};
}
