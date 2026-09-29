import { defineConfig } from '@doki-land/nifty';

/**
 * Nifty 工作区配置。
 *
 * Trusted Publisher 合同（与 `.github/workflows/publish-npm.yml` 一致）：
 * - file = `publish-npm.yml`
 * - env  = `NPM_PUBLISH`
 * - repo = `valkyrie-language/valkyrie.rs`（通过 `NIFTY_TRUST_REPO`，见 `scripts/run-nifty.mjs`）
 *
 * 本地密钥：`.env.placeholder.local`（nifty 原生）或兼容 `.env.npm-trust.local`。
 */
export default defineConfig({
    authorMap: 'documentation/maintenance/author-github.json',
    changelog: {
        repo: 'valkyrie-language/valkyrie.rs',
        releasesDir: 'documentation/maintenance/releases',
    },
    publish: {
        packages: [
            '@valkyrie-language/vcc',
            '@valkyrie-language/vcc-win32-x64',
            '@valkyrie-language/vcc-linux-x64',
            '@valkyrie-language/vcc-darwin-x64',
            '@valkyrie-language/vcc-darwin-arm64',
            '@valkyrie-language/vcc-unknown-wasm32',
            '@valkyrie-language/vcc-wasm32-wasi',
            '@valkyrie-language/legion',
            '@valkyrie-language/asgard',
        ],
    },
    format: {
        preset: 'default',
        style: {
            indentStyle: 'space',
            indentWidth: 4,
            lineWidth: 144,
            quoteStyle: 'single',
        },
    },
});
