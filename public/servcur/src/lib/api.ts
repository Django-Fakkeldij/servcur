export const API_HOST = '127.0.0.1:3000' as const;
export const API_URL = `http://${API_HOST}` as const;
export const API_WS_URL = `ws://${API_HOST}` as const;

export const API_ROUTES = {
	system: `${API_URL}/system` as const,
	projects: `${API_URL}/projects` as const,
	containers: `${API_URL}/containers` as const,
	containers_logs_ws: (name: string, since: number) => `${API_WS_URL}/containers/${name}/logs?since=${since}` as const,
	container_remove: (name: string) => `${API_URL}/containers/${name}/remove` as const,
	container_start: (name: string) => `${API_URL}/containers/${name}/start` as const,
	container_restart: (name: string) => `${API_URL}/containers/${name}/restart` as const,
	container_stop: (name: string) => `${API_URL}/containers/${name}/stop` as const,
	images: `${API_URL}/images` as const,
	images_prune: `${API_URL}/images/prune` as const,
	image_remove: (name: string) => `${API_URL}/images/${name}/remove` as const,
	volumes: `${API_URL}/volumes` as const,
	volumes_prune: `${API_URL}/volumes/prune` as const,
	volume_remove: (name: string) => `${API_URL}/volumes/${name}/remove` as const,
	networks: `${API_URL}/networks` as const,
	networks_prune: `${API_URL}/networks/prune` as const,
	network_remove: (name: string) => `${API_URL}/networks/${name}/remove` as const,
} as const;
