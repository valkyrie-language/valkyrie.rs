import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.join(__dirname, '..');

const filesToRemoveTests = [
    { file: 'projects/valkyrie-compiler/src/flags_gen.rs', testStartLine: 884 },
    { file: 'projects/valkyrie-compiler/src/module/graph.rs', testStartLine: 223 },
    { file: 'projects/valkyrie-compiler/src/module/mod.rs', testStartLine: 398 },
    { file: 'projects/valkyrie-compiler/src/pipeline/ast_to_hir.rs', testStartLine: 1123 },
    { file: 'projects/valkyrie-compiler/src/pipeline/derive_injection.rs', testStartLine: 124 },
    { file: 'projects/valkyrie-compiler/src/trait_resolver.rs', testStartLine: 904 },
    { file: 'projects/valkyrie-compiler/src/type_checker/abstract_class.rs', testStartLine: 502 },
    { file: 'projects/valkyrie-compiler/src/type_checker/constraint_solver.rs', testStartLine: 2370 },
    { file: 'projects/valkyrie-compiler/src/type_checker/inference.rs', testStartLine: 2658 },
    { file: 'projects/valkyrie-compiler/src/type_checker/sealed_class.rs', testStartLine: 160 },
    { file: 'projects/valkyrie-compiler/src/type_checker/value_type.rs', testStartLine: 536 },
    { file: 'projects/valkyrie-compiler/src/type_checker/widget.rs', testStartLine: 469 },
    { file: 'projects/valkyrie-compiler/src/typing/escape_analysis.rs', testStartLine: 736 },
    { file: 'projects/valkyrie-compiler/src/typing/mro.rs', testStartLine: 234 },
    { file: 'projects/valkyrie-compiler/src/visibility.rs', testStartLine: 178 },
    { file: 'projects/valkyrie-compiler/src/widget_runtime.rs', testStartLine: 1210 },
    { file: 'projects/valkyrie-compiler/src/witness_linker.rs', testStartLine: 511 },
    { file: 'projects/valkyrie-compiler/src/witness_serde.rs', testStartLine: 311 },
    { file: 'projects/valkyrie-testing/src/lib.rs', testStartLine: 343 },
];

function removeTestModule(filePath, testStartLine) {
    const fullPath = path.join(ROOT_DIR, filePath);
    const content = fs.readFileSync(fullPath, 'utf-8');
    const lines = content.split('\n');
    
    const testLineIndex = testStartLine - 1;
    const lineContent = lines[testLineIndex];
    
    if (!lineContent || !lineContent.includes('#[cfg(test)]')) {
        console.log(`SKIP: ${filePath} - Line ${testStartLine} does not contain #[cfg(test)]`);
        return false;
    }
    
    let endLineIndex = testLineIndex;
    let braceCount = 0;
    let foundOpeningBrace = false;
    
    for (let i = testLineIndex; i < lines.length; i++) {
        const line = lines[i];
        
        for (const char of line) {
            if (char === '{') {
                braceCount++;
                foundOpeningBrace = true;
            } else if (char === '}') {
                braceCount--;
            }
        }
        
        if (foundOpeningBrace && braceCount === 0) {
            endLineIndex = i;
            break;
        }
    }
    
    let newLines = lines.slice(0, testLineIndex);
    
    while (newLines.length > 0) {
        const lastLine = newLines[newLines.length - 1];
        if (lastLine.trim() === '') {
            newLines.pop();
        } else {
            break;
        }
    }
    
    let newContent = newLines.join('\n');
    if (!newContent.endsWith('\n')) {
        newContent += '\n';
    }
    
    fs.writeFileSync(fullPath, newContent, 'utf-8');
    console.log(`OK: ${filePath} - Removed lines ${testStartLine}-${endLineIndex + 1}`);
    return true;
}

console.log('Removing test modules from source files...\n');

let successCount = 0;
let skipCount = 0;

for (const { file, testStartLine } of filesToRemoveTests) {
    const result = removeTestModule(file, testStartLine);
    if (result) {
        successCount++;
    } else {
        skipCount++;
    }
}

console.log(`\nDone! Success: ${successCount}, Skipped: ${skipCount}`);
