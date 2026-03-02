import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

function getRootDirectoryPath() {
	return path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
}

function isPlainObject(value) {
	return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function readOptionalStringProperty(objectValue, propertyName) {
	if (!isPlainObject(objectValue)) {
		return undefined;
	}
	const propertyValue = objectValue[propertyName];
	return typeof propertyValue === 'string' ? propertyValue : undefined;
}

async function readAppVersion(rootDir) {
	const packageJsonPath = path.join(rootDir, 'package.json');
	const raw = await readFile(packageJsonPath, { encoding: 'utf8' });
	const parsed = JSON.parse(raw);
	if (!isPlainObject(parsed)) {
		throw new Error('package.json must be an object');
	}
	const version = readOptionalStringProperty(parsed, 'version');
	if (typeof version !== 'string' || version.trim().length === 0) {
		throw new Error('package.json version must be a non-empty string');
	}
	return version;
}

function updateCargoTomlContent(currentContent, version) {
	const lines = currentContent.split(/\r?\n/);
	let inPackageSection = false;
	let didUpdate = false;

	const nextLines = lines.map((line) => {
		const headerMatch = line.match(/^\s*\[(.+?)\]\s*$/);
		if (headerMatch !== null) {
			inPackageSection = headerMatch[1] === 'package';
			return line;
		}

		if (!inPackageSection) {
			return line;
		}

		const versionMatch = line.match(/^(\s*version\s*=\s*)"[^"]*"(\s*)$/);
		if (versionMatch === null) {
			return line;
		}

		didUpdate = true;
		return `${versionMatch[1]}"${version}"${versionMatch[2]}`;
	});

	if (!didUpdate) {
		throw new Error('Failed to update Cargo.toml: [package].version not found');
	}

	return nextLines.join('\n');
}

async function syncCargoToml(rootDir, version) {
	const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');
	const current = await readFile(cargoTomlPath, { encoding: 'utf8' });
	const next = updateCargoTomlContent(current, version);
	if (next === current) {
		return false;
	}
	await writeFile(cargoTomlPath, next, { encoding: 'utf8' });
	return true;
}

async function syncTauriConfig(rootDir, version) {
	const tauriConfigPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
	const raw = await readFile(tauriConfigPath, { encoding: 'utf8' });
	const parsed = JSON.parse(raw);
	if (!isPlainObject(parsed)) {
		throw new Error('tauri.conf.json must be an object');
	}
	const currentVersion = readOptionalStringProperty(parsed, 'version');
	if (typeof currentVersion !== 'string') {
		throw new Error('tauri.conf.json must contain a string version field');
	}
	if (currentVersion === version) {
		return false;
	}
	const nextConfig = { ...parsed, version };
	await writeFile(tauriConfigPath, `${JSON.stringify(nextConfig, null, '\t')}\n`, { encoding: 'utf8' });
	return true;
}

async function main() {
	const rootDir = getRootDirectoryPath();
	const version = await readAppVersion(rootDir);

	const [didUpdateCargo, didUpdateTauri] = await Promise.all([
		syncCargoToml(rootDir, version),
		syncTauriConfig(rootDir, version),
	]);

	if (didUpdateCargo || didUpdateTauri) {
		process.stdout.write(`Synced app version to ${version}\n`);
	}
}

main().catch((error) => {
	const message = error instanceof Error ? error.message : String(error);
	process.stderr.write(`${message}\n`);
	process.exitCode = 1;
});
