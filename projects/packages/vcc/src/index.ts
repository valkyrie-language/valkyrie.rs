export {
    NATIVE_PACKAGES,
    NATIVE_PACKAGE_TRIPLES,
    resolveNativeNpmTriple,
    nativeCollectBinaryName,
    locateNativeCollect,
    resolveWasmMjs,
    loadNativeBindingAt,
    loadNativeBinding,
    planProject,
    buildProject,
    spyRun,
    type VccNativeBinding,
    type VccNativeLegionOutcome,
    type VccSpyTargetRunOptions,
    type LegionPlanOutcome,
    type LegionBuildOutcome,
} from './native.ts';

export {
    DEFAULT_SPAWN_MAX_BUFFER_BYTES,
    DEFAULT_SPAWN_TIMEOUT_MS,
    legionBuildOutcomeToSpawnResult,
    legionPlanOutcomeToSpawnResult,
    spawnWasmCollect,
    type SpawnWasmCollectOptions,
    type VccCliRoute,
    type VccCliSpawnResult,
} from './spawn-wasm.ts';
