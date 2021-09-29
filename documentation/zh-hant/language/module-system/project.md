# Project 模式

## 概述

Project 模式是 Valkyrie 語言的單專案管理模式，透過 `legion.json` 檔案定義專案設定。當目錄中存在 `legion.json` 檔案且沒有 `legions.json` 檔案時，該目錄被識別為獨立專案模式。

## 檔案結構

```
project-root/
├── legion.json           # 專案設定檔
├── library/
│   └── _.vk             # 建議：命名空間根目錄定義（也支援 _.valkyrie）
├── binary/
│   ├── main.vk          # 註冊為指令 `main`
│   ├── tool1.vk         # 註冊為指令 `tool1`
│   └── tool2/
│       └── _.vk         # 註冊為指令 `tool2`
├── tests/               # 測試檔案
├── docs/                # 文件目錄
├── examples/            # 範例程式碼
└── target/              # 建置輸出
```

## legion.json 設定

`legion.json` 是 JSON5 格式的設定檔，支援註釋和更靈活的語法：

### 基礎設定

```json5
{
    // 專案基本資訊
    "name": "my-valkyrie-project",
    "version": "1.0.0",
    "description": "A Valkyrie language project",
    "type": "application", // 或 "library", "plugin", "tool"
    
    // 作者資訊
    "authors": [
        "Developer Name <email@example.com>"
    ],
    
    // 授權條款
    "license": "MIT",
    
    // 儲存庫資訊
    "repository": "https://github.com/user/my-valkyrie-project",
    "homepage": "https://my-project.dev",
    "documentation": "https://docs.my-project.dev",
    
    // 發布設定
    "publish": true,
    "private": false,
    
    // 語言版本
    "edition": "2024",
    
    // 入口檔案
    "main": "binary/main.vk",
    "lib": "library/_.vk"
}
```

### 相依性管理

```json5
{
    // 執行階段相依性
    "dependencies": {
        "valkyrie-std": "^1.0.0",
        "serde": {
            "version": "^1.0",
            "features": ["derive"]
        },
        "local-lib": {
            "path": "../local-lib"
        },
        "git-dep": {
            "git": "https://github.com/user/repo.git",
            "branch": "main"
        }
    },
    
    // 建置時相依性
    "build-dependencies": {
        "build-script": "^0.1.0"
    },
    
    // 開發相依性
    "dev-dependencies": {
        "test-framework": "^2.0.0",
        "benchmark": "^1.5.0"
    },
    
    // 可選相依性
    "optional-dependencies": {
        "feature-x": "^1.0.0",
        "feature-y": "^2.0.0"
    }
}
```

### 功能特性

```json5
{
    // 功能特性定義
    "features": {
        "default": ["std", "serde"],
        "std": [],
        "serde": ["dep:serde"],
        "async": ["dep:tokio"],
        "full": ["std", "serde", "async"]
    }
}
```

### 建置設定

```json5
{
    // 建置設定
    "build": {
        "target": "native", // 或 "wasm", "js", "llvm"
        "optimization": "release", // 或 "debug", "size", "speed"
        "output": "target/",
        "incremental": true,
        "parallel": true,
        "cache": true
    },
    
    // 編譯器選項
    "compiler": {
        "warnings": "deny",
        "errors": "abort",
        "debug-info": true,
        "strip-symbols": false
    },
    
    // 連結器選項
    "linker": {
        "lto": true,
        "strip": false,
        "static": false
    }
}
```

### 腳本命令

```json5
{
    // 自定義腳本
    "scripts": {
        "build": "valkyrie build --release",
        "test": "valkyrie test",
        "run": "valkyrie run",
        "clean": "valkyrie clean",
        "fmt": "valkyrie fmt",
        "lint": "valkyrie lint",
        "doc": "valkyrie doc --open",
        "publish": "valkyrie publish",
        "install": "valkyrie install",
        "dev": "valkyrie run --watch",
        "benchmark": "valkyrie bench"
    }
}
```

## 專案類型

### 1. 應用程式 (Application)

```json5
{
    "type": "application",
    "main": "binary/main.vk",
    "build": {
        "target": "native",
        "executable": "my-app"
    }
}
```

### 2. 函式庫 (Library)

```json5
{
    "type": "library",
    "lib": "library/_.vk",
    "build": {
        "target": "library",
        "crate-type": ["lib", "dylib"]
    }
}
```

### 3. 插件 (Plugin)

```json5
{
    "type": "plugin",
    "plugin": {
        "interface": "valkyrie-plugin-api",
        "version": "1.0.0"
    }
}
```

### 4. 工具 (Tool)

```json5
{
    "type": "tool",
    "bin": [
        {
            "name": "my-tool",
            "path": "src/bin/tool.vk"
        }
    ]
}
```

## 語義特性

### 1. 相依性解析

- **版本約束**：支援語義化版本約束
- **路徑相依**：支援本地路徑相依
- **Git 相依**：支援 Git 儲存庫相依
- **條件相依**：基於特性的條件相依

### 2. 特性系統

- **預設特性**：自動啟用的特性集合
- **可選特性**：按需啟用的功能特性
- **特性組合**：特性之間的相依關係
- **特性傳播**：相依項目特性的傳播機制

### 3. 建置系統

- **增量編譯**：只編譯變更的部分
- **並行建置**：多核並行編譯
- **快取機制**：建置結果快取
- **交叉編譯**：支援多目標平台編譯

### 4. 套件管理

- **版本管理**：自動版本號管理
- **發布流程**：標準化的發布流程
- **相依性鎖定**：確保建置的可重現性
- **安全檢查**：相依性安全性檢查

## 開發工作流

### 1. 專案初始化

```bash
# 建立新專案
valkyrie new my-project
cd my-project

# 或者初始化現有目錄
valkyrie init
```

### 2. 相依性管理

```bash
# 新增相依性
valkyrie add serde@^1.0

# 新增開發相依性
valkyrie add --dev test-framework

# 更新相依性
valkyrie update

# 移除相依性
valkyrie remove old-dep
```

### 3. 建置和測試

```bash
# 建置專案
valkyrie build

# 執行專案
valkyrie run

# 執行測試
valkyrie test

# 產生文件
valkyrie doc

# 執行二進位程式
v main                     # 執行 binary/main.vk
v tool1                    # 執行 binary/tool1.vk
v tool2                    # 執行 binary/tool2/_.vk
```

### 4. 發布流程

```bash
# 檢查專案
valkyrie check

# 執行完整測試
valkyrie test --all-features

# 發布到儲存庫
valkyrie publish
```

## 設定範例

### Web 應用程式專案

```json5
{
    "name": "web-app",
    "version": "1.0.0",
    "type": "application",
    "main": "src/main.vk",
    
    "dependencies": {
        "valkyrie-web": "^2.0.0",
        "valkyrie-router": "^1.5.0",
        "valkyrie-templates": "^1.0.0"
    },
    
    "features": {
        "default": ["server"],
        "server": ["dep:valkyrie-web"],
        "client": ["dep:valkyrie-wasm"]
    },
    
    "build": {
        "target": "wasm",
        "optimization": "size"
    },
    
    "scripts": {
        "dev": "valkyrie run --watch --features server",
        "build-client": "valkyrie build --target wasm --features client",
        "serve": "valkyrie run --release"
    }
}
```

### 函式庫專案

```json5
{
    "name": "data-structures",
    "version": "2.1.0",
    "type": "library",
    "lib": "src/lib.vk",
    "description": "High-performance data structures for Valkyrie",
    
    "authors": ["Library Team <team@example.com>"],
    "license": "Apache-2.0",
    "repository": "https://github.com/team/data-structures",
    
    "dependencies": {
        "valkyrie-std": "^1.0.0"
    },
    
    "dev-dependencies": {
        "criterion": "^0.5.0",
        "proptest": "^1.0.0"
    },
    
    "features": {
        "default": ["std"],
        "std": [],
        "no-std": [],
        "serde": ["dep:serde"],
        "parallel": ["dep:rayon"]
    },
    
    "build": {
        "optimization": "speed",
        "lto": true
    }
}
```

## 最佳實踐

### 1. 專案結構

- **清晰分層**：合理組織原始碼結構
- **模組化設計**：使用模組系統組織程式碼
- **測試覆蓋**：為所有公開 API 編寫測試
- **文件完整**：提供完整的 API 文件

### 2. 相依性管理

- **最小相依**：只新增必要的相依性
- **版本鎖定**：使用精確的版本約束
- **定期更新**：定期更新相依性版本
- **安全稽核**：定期進行安全稽核

### 3. 建置優化

- **增量建置**：利用增量編譯特性
- **並行建置**：啟用並行編譯
- **快取利用**：充分利用建置快取
- **目標優化**：針對目標平台優化

### 4. 版本管理

- **語義化版本**：遵循語義化版本規範
- **變更日誌**：維護詳細的變更日誌
- **回溯相容**：保持 API 的回溯相容性
- **棄用策略**：合理的 API 棄用策略

## 工具整合

### IDE 支援

- **專案識別**：IDE 自動識別專案設定
- **相依性管理**：圖形化的相依性管理介面
- **建置整合**：整合的建置和執行功能
- **除錯支援**：完整的除錯功能支援

### CI/CD 整合

- **自動建置**：基於設定的自動建置
- **測試執行**：自動化測試執行
- **品質檢查**：程式碼品質和安全檢查
- **自動發布**：基於標籤的自動發布

## 遷移和相容性

### 設定遷移

- **版本升級**：設定檔版本升級
- **格式轉換**：從其他格式轉換
- **回溯相容**：保持回溯相容性
- **遷移工具**：提供自動遷移工具

### 生態系統相容

- **標準遵循**：遵循社群標準
- **工具鏈整合**：與現有工具鏈整合
- **平台支援**：多平台支援
- **互操作性**：與其他語言的互操作
