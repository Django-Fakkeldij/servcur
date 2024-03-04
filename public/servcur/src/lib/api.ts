export const API_HOST = '127.0.0.1:3000' as const;
export const API_URL = `http://${API_HOST}` as const;
export const API_WS_URL = `ws://${API_HOST}` as const;

export const API_ROUTES = {
	system: `${API_URL}/system` as const,
	containers: `${API_URL}/containers` as const,
	containers_logs_ws: (name: string, since: number) => `${API_WS_URL}/containers/${name}/logs?since=${since}` as const,
	images: `${API_URL}/images` as const,
	volumes: `${API_URL}/volumes` as const,
	networks: `${API_URL}/networks` as const,
} as const;
