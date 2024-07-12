import { base } from '$app/paths';

export const routes = {
	home: `${base}/` as const,
	dashboard: `${base}/dashboard` as const,
	projects: `${base}/projects` as const,
	project_create: `${base}/projects/create` as const,
	containers: `${base}/containers` as const,
	container: (id: string) => `${base}/containers/${id}` as const,
	project: (name: string, branch: string) => `${base}/projects/${name}/${branch}` as const,
	images: `${base}/images` as const,
	volumes: `${base}/volumes` as const,
	networks: `${base}/networks` as const,
} as const;
