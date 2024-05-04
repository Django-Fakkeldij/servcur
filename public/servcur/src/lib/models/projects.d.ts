export interface ProjectGet {
	branch: string;
	path: string;
	project_kind: ProjectKind;
	project_name: string;
	uri: string;
}

export type ProjectKind = {
	type: 'DockerFile';
	image_version: number;
};

export type DockerFileCommands = 'Build' | 'Start' | 'Stop';

export interface ProjectActionReturn {
	project: {
		name: string;
		branch: string;
	};
	io_id: string;
}
