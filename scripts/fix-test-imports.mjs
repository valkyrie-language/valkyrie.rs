import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.join(__dirname, '..');
const TESTS_DIR = path.join(ROOT_DIR, 'projects', 'valkyrie-compiler', 'tests');

const commonImports = {
    'Identifier': ['use valkyrie_types::Identifier;'],
    'SourceSpan': ['use valkyrie_types::SourceSpan;'],
    'SourceID': ['use valkyrie_types::SourceID;'],
    'NamePath': ['use valkyrie_types::NamePath;'],
    'HirVisibility': ['use valkyrie_types::hir::HirVisibility;'],
    'HirDocumentation': ['use valkyrie_types::hir::HirDocumentation;'],
    'HirType': ['use valkyrie_types::hir::HirType;'],
    'HirExpr': ['use valkyrie_types::hir::HirExpr;'],
    'HirExprKind': ['use valkyrie_types::hir::HirExprKind;'],
    'HirLiteral': ['use valkyrie_types::hir::HirLiteral;'],
    'HirFunction': ['use valkyrie_types::hir::HirFunction;'],
    'HirStruct': ['use valkyrie_types::hir::HirStruct;'],
    'HirBlock': ['use valkyrie_types::hir::HirBlock;'],
    'HirParam': ['use valkyrie_types::hir::HirParam;'],
    'HirGeneric': ['use valkyrie_types::hir::HirGeneric;'],
    'HirField': ['use valkyrie_types::hir::HirField;'],
    'HirWidget': ['use valkyrie_types::hir::HirWidget;'],
    'HirWidgetLifecycle': ['use valkyrie_types::hir::HirWidgetLifecycle;'],
    'HirBinaryOp': ['use valkyrie_types::hir::HirBinaryOp;'],
    'HirUnaryOp': ['use valkyrie_types::hir::HirUnaryOp;'],
    'HirIdentifier': ['use valkyrie_types::hir::HirIdentifier;'],
    'HirModule': ['use valkyrie_types::hir::HirModule;'],
    'HirCapture': ['use valkyrie_types::hir::HirCapture;'],
    'HirPattern': ['use valkyrie_types::hir::HirPattern;'],
    'HirStatement': ['use valkyrie_types::hir::HirStatement;'],
    'HirStatementKind': ['use valkyrie_types::hir::HirStatementKind;'],
    'HirAssociatedType': ['use valkyrie_types::hir::HirAssociatedType;'],
    'HirAssociatedTypeImpl': ['use valkyrie_types::hir::HirAssociatedTypeImpl;'],
    'CaptureStorage': ['use valkyrie_types::hir::CaptureStorage;'],
    'CaptureStorageInfo': ['use valkyrie_types::hir::CaptureStorageInfo;'],
    'CaptureOptimization': ['use valkyrie_types::hir::CaptureOptimization;'],
    'EscapeInfo': ['use valkyrie_types::hir::EscapeInfo;'],
    'EscapeKind': ['use valkyrie_types::hir::EscapeKind;'],
    'EscapeReason': ['use valkyrie_types::hir::EscapeReason;'],
    'HirFlags': ['use valkyrie_types::hir::HirFlags;'],
    'HirFlagMember': ['use valkyrie_types::hir::HirFlagMember;'],
    'TypeMetadata': ['use valkyrie_types::witness::TypeMetadata;'],
    'TypeId': ['use valkyrie_types::witness::TypeId;'],
    'ModuleId': ['use valkyrie_types::witness::ModuleId;'],
    'TraitId': ['use valkyrie_types::witness::TraitId;'],
    'MethodId': ['use valkyrie_types::witness::MethodId;'],
    'MethodPath': ['use valkyrie_types::witness::MethodPath;'],
    'MethodEntry': ['use valkyrie_types::witness::MethodEntry;'],
    'WitnessTable': ['use valkyrie_types::witness::WitnessTable;'],
    'WitnessTableBuilder': ['use valkyrie_types::witness::WitnessTableBuilder;'],
    'WitnessRegistry': ['use valkyrie_types::witness::WitnessRegistry;'],
    'CrossModuleError': ['use valkyrie_types::witness::CrossModuleError;'],
    'TypeKind': ['use valkyrie_types::witness::TypeKind;'],
    'WITNESS_MAGIC': ['use valkyrie_types::witness::WITNESS_MAGIC;'],
    'WITNESS_VERSION': ['use valkyrie_types::witness::WITNESS_VERSION;'],
    'QualifiedName': ['use nyar_types::QualifiedName;'],
};

const filesToFix = [
    'flags_gen.rs',
    'trait_resolver.rs',
    'visibility.rs',
    'witness_linker.rs',
    'witness_serde.rs',
    'widget_runtime.rs',
    'typing/escape_analysis.rs',
    'typing/mro.rs',
    'type_checker/abstract_class.rs',
    'type_checker/constraint_solver.rs',
    'type_checker/inference.rs',
    'type_checker/sealed_class.rs',
    'type_checker/value_type.rs',
    'type_checker/widget.rs',
    'pipeline/ast_to_hir.rs',
    'pipeline/derive_injection.rs',
    'module/graph.rs',
    'module/module_error.rs',
    'codegen/renamed_inheritance.rs',
];

function findUsedTypes(content) {
    const types = new Set();
    for (const type of Object.keys(commonImports)) {
        const patterns = [
            new RegExp(`\\b${type}\\b(?![^\\n]*use\\s)`),
            new RegExp(`:\\s*${type}\\b`),
            new RegExp(`->\\s*${type}\\b`),
            new RegExp(`<${type}\\b`),
            new RegExp(`\\(${type}\\b`),
            new RegExp(`\\[${type}\\b`),
            new RegExp(`=\\s*${type}\\b`),
            new RegExp(`\\b${type}::`),
        ];
        for (const pattern of patterns) {
            if (pattern.test(content)) {
                types.add(type);
                break;
            }
        }
    }
    return types;
}

function fixFileImports(filePath) {
    const fullPath = path.join(TESTS_DIR, filePath);
    if (!fs.existsSync(fullPath)) {
        console.log(`SKIP: ${filePath} - File not found`);
        return false;
    }

    let content = fs.readFileSync(fullPath, 'utf-8');
    const lines = content.split('\n');
    
    const existingImports = new Set();
    let importEndIndex = lines.findIndex(line => {
        const trimmed = line.trim();
        return !trimmed.startsWith('use ') && 
               !trimmed.startsWith('mod ') && 
               trimmed !== '' && 
               !trimmed.startsWith('//') && 
               !trimmed.startsWith('/*') &&
               !trimmed.startsWith('extern ');
    });
    
    if (importEndIndex === -1) {
        importEndIndex = 0;
    }
    
    for (const line of lines.slice(0, importEndIndex)) {
        existingImports.add(line.trim());
    }
    
    const usedTypes = findUsedTypes(content);
    const neededImports = [];
    
    for (const type of usedTypes) {
        const imports = commonImports[type];
        if (imports) {
            for (const imp of imports) {
                if (!existingImports.has(imp.trim())) {
                    neededImports.push(imp);
                    existingImports.add(imp.trim());
                }
            }
        }
    }
    
    if (neededImports.length === 0) {
        console.log(`OK: ${filePath} - No new imports needed`);
        return false;
    }
    
    const uniqueImports = [...new Set(neededImports)].sort();
    
    const beforeImports = lines.slice(0, importEndIndex);
    const afterImports = lines.slice(importEndIndex);
    
    const newContent = [...beforeImports, ...uniqueImports, '', ...afterImports].join('\n');
    fs.writeFileSync(fullPath, newContent, 'utf-8');
    
    console.log(`FIXED: ${filePath} - Added ${uniqueImports.length} imports`);
    return true;
}

console.log('Fixing test file imports...\n');

let fixedCount = 0;
for (const file of filesToFix) {
    if (fixFileImports(file)) {
        fixedCount++;
    }
}

console.log(`\nDone! Fixed ${fixedCount} files.`);
