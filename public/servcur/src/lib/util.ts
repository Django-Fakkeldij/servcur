export function capatalizeWord(inp: string | undefined): string {
	if (!inp || inp.length < 1) return '';
	if (inp.length === 1) return inp.toLocaleUpperCase();

	return inp[0].toLocaleUpperCase() + inp.slice(1);
}
