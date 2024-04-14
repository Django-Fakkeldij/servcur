export const routes = {
	home: '/' as const,
	dashboard: '/dashboard' as const,
	projects: '/projects' as const,
	containers: '/containers' as const,
	container: (id: string) => `/containers/${id}` as const,
	images: '/images' as const,
	volumes: '/volumes' as const,
	networks: '/networks' as const,
} as const;
