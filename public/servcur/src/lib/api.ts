export const API_URL = 'http://127.0.0.1:3000' as const;

export const API_ROUTES = {
	containers: `${API_URL}/containers` as const,
	images: `${API_URL}/images` as const,
	volumes: `${API_URL}/volumes` as const,
	networks: `${API_URL}/networks` as const,
} as const;
