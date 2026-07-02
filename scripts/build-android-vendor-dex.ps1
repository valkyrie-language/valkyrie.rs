# 维护者脚本：产出 vendor/android-compose/dex/compose-shell.dex
#
# 重要说明（避免误解）：
# - 普通用户 / 项目使用者：`asgard build` / `asgard pack` 不需要 ANDROID_HOME / KOTLIN_HOME，
#   也不会调用本脚本；缺 vendor 时会自动走 Rust bootstrap（门禁/CI 形态），仍可打包出 APK。
# - 维护者（要生成“真机可运行的 Compose Vendor dex”）：需要 Android SDK（d8）+ Kotlin 编译器（kotlinc），
#   因此才需要配置 ANDROID_HOME / KOTLIN_HOME，并准备 vendor/android-compose/libs/*.jar。
param(
    [string]$AndroidHome = $env:ANDROID_HOME,
    [string]$OutDir = "$PSScriptRoot/../vendor/android-compose"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path "$PSScriptRoot/.."
Push-Location $RepoRoot

function Write-BootstrapDex {
    Write-Host "[bootstrap] 写入 Rust stub compose-shell.dex（用于门禁/CI；非完整 AndroidX Compose）..."
    cargo test -p voa --lib bootstrap_vendor_dex -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw "bootstrap_vendor_dex failed" }
}

function Write-KotlinSources {
    Write-Host "[sdk] 写出 Kotlin 源到 $OutDir/build/kotlin ..."
    cargo test -p voa --lib write_kotlin_sources_for_vendor_build -- --ignored --nocapture
    if ($LASTEXITCODE -ne 0) { throw "write_kotlin_sources failed" }
}

function Try-SdkCompile {
    param([string]$Home)
    $buildTools = Get-ChildItem -Path "$Home/build-tools" -Directory -ErrorAction SilentlyContinue |
        Sort-Object Name -Descending | Select-Object -First 1
    if (-not $buildTools) {
        Write-Warning "未找到 build-tools"
        return $false
    }
    $d8 = Join-Path $buildTools.FullName "d8.bat"
    if (-not (Test-Path $d8)) {
        Write-Warning "未找到 d8: $d8"
        return $false
    }
    $kotlinSrc = Join-Path $OutDir "build/kotlin/AsgardRuntime.kt"
    if (-not (Test-Path $kotlinSrc)) {
        Write-Warning "缺少 Kotlin 源: $kotlinSrc"
        return $false
    }
    $libsDir = Join-Path $OutDir "libs"
    if (-not (Test-Path $libsDir)) {
        Write-Warning "缺少 $libsDir（请放入 Compose/Activity/Material3 AAR 解压的 classes.jar）"
        return $false
    }
    $jars = Get-ChildItem -Path $libsDir -Filter "*.jar" -Recurse
    if ($jars.Count -eq 0) {
        Write-Warning "libs/ 下无 jar"
        return $false
    }
    $ktHome = $env:KOTLIN_HOME
    if (-not $ktHome) {
        Write-Warning "KOTLIN_HOME 未设置，跳过 kotlinc"
        return $false
    }
    $kotlinc = Join-Path $ktHome "bin/kotlinc.bat"
    if (-not (Test-Path $kotlinc)) {
        Write-Warning "未找到 kotlinc: $kotlinc"
        return $false
    }

    $classesOut = Join-Path $OutDir "build/classes"
    $dexOut = Join-Path $OutDir "dex"
    New-Item -ItemType Directory -Force -Path $classesOut, $dexOut | Out-Null
    $cp = ($jars | ForEach-Object { $_.FullName }) -join ";"
    & $kotlinc -jvm-target 1.8 -classpath $cp -d $classesOut $kotlinSrc
    if ($LASTEXITCODE -ne 0) { return $false }

    $classFiles = Get-ChildItem -Path $classesOut -Filter "*.class" -Recurse
    $inputs = @($classFiles.FullName) + @($jars.FullName)
    $shellDex = Join-Path $dexOut "compose-shell.dex"
    & $d8 --output $dexOut --min-api 24 @inputs
    if ($LASTEXITCODE -ne 0) { return $false }
    if (-not (Test-Path $shellDex)) {
        Write-Warning "d8 未生成 compose-shell.dex"
        return $false
    }

    $hash = (Get-FileHash $shellDex -Algorithm SHA256).Hash.ToLower()
    Write-Host "SDK compose-shell.dex sha256=$hash"
    $report = @{
        mode = "sdk"
        sha256 = $hash
        class_defs_hint = "run platform_contract vendor gate"
    } | ConvertTo-Json
    Set-Content -Path (Join-Path $OutDir "build-report.json") -Value $report
    return $true
}

try {
    New-Item -ItemType Directory -Force -Path "$OutDir/dex", "$OutDir/build" | Out-Null

    if ($AndroidHome -and (Test-Path $AndroidHome)) {
        Write-Host "[sdk] ANDROID_HOME=$AndroidHome"
        Write-KotlinSources
        if (Try-SdkCompile -Home $AndroidHome) {
            Write-Host "[sdk] 已生成 SDK vendor compose-shell.dex（用于真机运行）"
            exit 0
        }
        Write-Host "[sdk] 编译未成功，回退 bootstrap（仍可用于门禁/CI）..."
    }
    else {
        Write-Host "[bootstrap] ANDROID_HOME 未设置；使用 Rust stub"
    }

    Write-BootstrapDex
    Write-Host "[bootstrap] 已生成 vendor/android-compose/dex/compose-shell.dex"
    exit 0
}
finally {
    Pop-Location
}
