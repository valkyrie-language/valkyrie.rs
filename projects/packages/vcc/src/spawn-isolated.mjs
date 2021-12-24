import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);

const nativePath = process.env.VCC_NATIVE_PATH;
if (!nativePath) {
    console.error('VCC_NATIVE_PATH is required');
    process.exit(2);
}

let argv = [];
try {
    argv = JSON.parse(process.argv[2] ?? '[]');
} catch (error) {
    console.error(`invalid argv JSON: ${error}`);
    process.exit(2);
}

if (!Array.isArray(argv) || !argv.every((item) => typeof item === 'string')) {
    console.error('argv must be a string array');
    process.exit(2);
}

const binding = require(nativePath);
if (typeof binding.legionRunCaptured !== 'function') {
    console.error('native binding missing legionRunCaptured');
    process.exit(2);
}

const outcome = binding.legionRunCaptured(['legion', ...argv]);
process.stdout.write(JSON.stringify(outcome));
