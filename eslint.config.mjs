import typescriptEslint from '@typescript-eslint/eslint-plugin';
import globals from 'globals';
import tsParser from '@typescript-eslint/parser';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import js from '@eslint/js';
import { FlatCompat } from '@eslint/eslintrc';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const compat = new FlatCompat({
	baseDirectory: __dirname,
	recommendedConfig: js.configs.recommended,
	allConfig: js.configs.all,
});

export default [
	{
		ignores: ['dist/**', 'node_modules/**', 'python/**', 'src-tauri/target/**'],
	},
	...compat.extends('eslint:recommended', 'plugin:@typescript-eslint/recommended', 'plugin:prettier/recommended'),
	{
		files: ['**/*.{js,jsx,ts,tsx,mjs,cjs}'],

		plugins: {
			'@typescript-eslint': typescriptEslint,
		},

		languageOptions: {
			globals: {
				...globals.browser,
				...globals.node,
				...globals.jest,
			},

			parser: tsParser,
			parserOptions: {
				ecmaFeatures: {
					jsx: true,
				},
			},
			ecmaVersion: 'latest',
			sourceType: 'module',
		},

		rules: {
			indent: 'off',
			'linebreak-style': ['error', 'unix'],
			quotes: 'off',
			semi: ['error', 'always'],
			'@typescript-eslint/explicit-function-return-type': 'error',
			'@typescript-eslint/typedef': 'error',
			'@typescript-eslint/no-explicit-any': 'error',
			'@typescript-eslint/explicit-module-boundary-types': 'off',
			'@typescript-eslint/no-unsafe-function-type': 'warn',
			'@typescript-eslint/explicit-member-accessibility': [
				'error',
				{
					accessibility: 'explicit',

					overrides: {
						accessors: 'explicit',
						constructors: 'no-public',
						methods: 'explicit',
						properties: 'no-public',
						parameterProperties: 'no-public',
					},
				},
			],
		},
	},
];
