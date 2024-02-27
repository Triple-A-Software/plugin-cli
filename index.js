#!/usr/bin/env node

import childProcess from "child_process";
import path from "node:path";
import fs from "node:fs";

const ARCH_MAPPING = {
	x64: "x86_64",
	arm64: "aarch64",
};
const OS_MAPPING = {
	win32: "windows",
	darwin: "darwin",
	linux: "linux",
};

function getOS() {
	const os = process.platform;
	if (os in OS_MAPPING) {
		return OS_MAPPING[os];
	} else {
		throw new Error(`Unsupported OS: ${os}`);
	}
}

function getArch() {
	const arch = process.arch;
	if (arch in ARCH_MAPPING) {
		return ARCH_MAPPING[arch];
	} else {
		throw new Error(`Unsupported arch: ${arch}`);
	}
}

function runCli(os, arch, args) {
	const cli = path.resolve("bin", `plugin-cli-${os}-${arch}`);
	if (!fs.existsSync(cli)) {
		throw new Error(`Unsupported platform and architecture: ${os} ${arch}`);
	}
	const command = `${cli} ${args.join(" ")}`;
	try {
		return childProcess.execSync(command, {
			stdio: "inherit",
		});
	} catch (e) {}
}

const os = getOS();
const arch = getArch();
const args = process.argv.slice(2);
runCli(os, arch, args);
