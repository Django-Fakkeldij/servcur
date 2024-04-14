export interface ProjectGet {
	branch: string;
	path: string;
	project_kind: ProjectKind;
	project_name: string;
	uri: string;
}

export type ProjectKind = {
	type: 'dockerfile';
	image_version: number;
};
