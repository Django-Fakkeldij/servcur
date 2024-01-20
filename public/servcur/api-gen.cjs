// eslint-disable-next-line @typescript-eslint/no-var-requires
const OpenAPI = require('openapi-typescript-codegen');

OpenAPI.generate({
	// WHYYYYYY does this work like this.....
	input: '../../../../src/lib/docker_types/v1.43.yaml',
	output: './src/lib/docker_types/__generated/',
	exportCore: false,
	exportModels: true,
	exportSchemas: false,
	exportServices: false,
});
