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
