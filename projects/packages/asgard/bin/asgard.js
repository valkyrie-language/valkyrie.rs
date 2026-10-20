#!/usr/bin/env node
import { runCli } from '../src/cli/cli.ts';

const code = await runCli();
process.exit(code);
