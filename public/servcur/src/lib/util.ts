export function capatalizeWord(inp: string | undefined): string {
	if (!inp || inp.length < 1) return '';
	if (inp.length === 1) return inp.toLocaleUpperCase();

	return inp[0].toLocaleUpperCase() + inp.slice(1);
}

const magnitude_mibis = {
	KiB: 1024 ** 1,
	MiB: 1024 ** 2,
	GiB: 1024 ** 3,
	TiB: 1024 ** 4,
	PiB: 1024 ** 5,
} as const;
export function fileSizeMagnitudeMibis(inp: number): [number, keyof typeof magnitude_mibis] {
	let guess: keyof typeof magnitude_mibis = 'KiB';

	for (const k in magnitude_mibis) {
		const key = k as keyof typeof magnitude_mibis;
		if (inp < magnitude_mibis[key]) return [inp / magnitude_mibis[guess], guess];
		guess = key;
	}

	return [inp / magnitude_mibis[guess], guess];
}

const magnitude_bytes = {
	KB: 1000 ** 1,
	MB: 1000 ** 2,
	GB: 1000 ** 3,
	TB: 1000 ** 4,
	PB: 1000 ** 5,
} as const;
export function fileSizeMagnitudeBytes(inp: number): [number, keyof typeof magnitude_bytes] {
	let guess: keyof typeof magnitude_bytes = 'KB';

	for (const k in magnitude_bytes) {
		const key = k as keyof typeof magnitude_bytes;
		if (inp < magnitude_bytes[key]) return [inp / magnitude_bytes[guess], guess];
		guess = key;
	}

	return [inp / magnitude_bytes[guess], guess];
}

export function dateString(d: Date): string {
	return new Intl.DateTimeFormat(undefined, {
		dateStyle: 'medium',
		timeStyle: 'long',
	}).format(d);
}

export function makeId(prepend: string, hashable: string) {
	const h = cyrb53_hash(hashable);
	return `${prepend}-${h}`;
}

// Random hash fn => https://stackoverflow.com/questions/7616461/generate-a-hash-from-string-in-javascript
const cyrb53_hash = (str: string, seed = 0) => {
	let h1 = 0xdeadbeef ^ seed,
		h2 = 0x41c6ce57 ^ seed;
	for (let i = 0, ch; i < str.length; i++) {
		ch = str.charCodeAt(i);
		h1 = Math.imul(h1 ^ ch, 2654435761);
		h2 = Math.imul(h2 ^ ch, 1597334677);
	}
	h1 = Math.imul(h1 ^ (h1 >>> 16), 2246822507);
	h1 ^= Math.imul(h2 ^ (h2 >>> 13), 3266489909);
	h2 = Math.imul(h2 ^ (h2 >>> 16), 2246822507);
	h2 ^= Math.imul(h1 ^ (h1 >>> 13), 3266489909);

	return 4294967296 * (2097151 & h2) + (h1 >>> 0);
};
