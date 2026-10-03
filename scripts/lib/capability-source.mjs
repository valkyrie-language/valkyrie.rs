import { statSync } from 'node:fs';
import { join } from 'node:path';

export function capabilitySourceProject(sourceRoot) {
    if (!sourceRoot) {
        throw new Error('capability 必须显式提供 valkyrie.v 源码根：--valkyrie-v <dir>');
    }
    const project = join(sourceRoot, 'projects/legion._/projects/legion.tools');
    const manifest = statSync(join(project, 'legion.von'), { throwIfNoEntry: false });
    if (!manifest?.isFile()) {
        throw new Error('完整 legion.tools 源项目缺少 legion.von；正式 capability gate 拒绝旧布局和 bootstrap fixture');
    }
    return project;
}
