export const load = ({ params }) => {
	return {
		name: params.name,
		branch: params.branch,
	};
};
