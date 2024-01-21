export const API_URL = 'http://127.0.0.1:3000' as const;

export const API_ROUTES = {
	containers: `${API_URL}/containers` as const,
	images: `${API_URL}/images` as const,
} as const;
