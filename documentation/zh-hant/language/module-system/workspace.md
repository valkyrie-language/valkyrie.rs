# Workspace 模式

## 概述

Workspace 模式是 Valkyrie 語言的多專案管理模式，透過 `legions.json` 檔案定義工作區設定。當專案根目錄存在 `legions.json` 檔案時，該目錄被識別為 workspace 模式。

## 檔案結構

```
workspace-root/
├── legions.json          # Workspace 設定檔
├── project-a/
│   ├── legion.json       # 專案 A 設定
│   ├── library/
│   │   └── _.vk         # 建議：命名空間根目錄定義
│   └── binary/
│       └── main.vk      # 註冊為指令 `main`
├── project-b/
│   ├── legion.json       # 專案 B 設定
│   ├── library/
│   │   └── _.vk         # 函式庫程式碼目錄
│   └── binary/
│       ├── tool1.vk     # 註冊為指令 `tool1`
│       └── tool2/
│           └── _.vk     # 註冊為指令 `tool2`
└── shared/
    └── common/
        └── _.vk
```

## legions.json 設定

`legions.json` 是 JSON5 格式的設定檔，支援註釋和更靈活的語法：

```json5
{
    // Workspace 基本資訊
    "name": "valkyrie-workspace",
    "version": "1.0.0",
    "description": "Valkyrie language workspace",
    
    // 私有工作區標識
    "private": true,
    
    // 成員專案列表
    "members": [
        "projects/*",
        "tools/build-tools"
    ],
    
    // 排除的目錄
    "exclude": [
        "legacy/*",
        "experiments/*",
        "temp/*"
    ],
    
    // 預設成員（用於快速建置）
    "default-members": [
        "projects/valkyrie-core",
        "projects/valkyrie-std"
    ],
    
    // Workspace 階層的腳本
    "scripts": {
        "build": "legion build --release",
        "test": "legion test --release",
        "fmt": "legion fmt --all",
        "clean": "legion clean",
        "publish": "git push && git push --tags --prune",
        "upgrade": "legion upgrade --workspace"
    },
    
    // 共享相依性設定
    "dependencies": {
        "shared": {
            "serde": "^1.0",
            "tokio": "^1.0"
        }
    },
    
    // 建置設定
    "build": {
        "profile": {
            "release": {
                "lto": true,
                "opt-level": "s"
            }
        }
    }
}
```

## 語義特性

### 1. 專案發現

- **自動發現**：根據 `members` 模式自動發現子專案
- **顯式排除**：透過 `exclude` 排除不需要的目錄

### 2. 屬性繼承 (Property Inheritance)

Workspace 支援自動屬性繼承機制。如果成員專案未顯式定義某些元數據，它們將自動從工作區根目錄的 `legions.json` 中繼承。

#### 工作區設定 (`legions.json`)
```json5
{
    "name": "my-workspace",
    "version": "1.2.3",
    "description": "A unified workspace",
    "authors": ["Valkyrie Team"],
    "license": "MIT",
    "members": ["projects/*"],
    "dependencies": {
        "shared_lib": "1.0.0"
    }
}
```

#### 成員專案設定 (`projects/my-pkg/legion.json`)
通用元數據（如版本、作者、授權條款等）是**自動繼承**的，無需額外設定：

```json5
{
    "name": "my-pkg",
    // version 自動繼承為 1.2.3
    // description 自動繼承為 "A unified workspace"
    
    "dependencies": {
        // 相依項目不會自動繼承，必須顯式聲明
        // 如果要使用工作區定義的版本，直接設為 true 即可
        "shared_lib": true
    }
}
```

自動繼承的屬性包括：
- `version`
- `description`
- `authors`
- `license`
- `repository`
- `homepage`
- `edition`

### 3. 相依性共享

- **直接引用**：工作區內的套件可以透過名稱直接 `using` 彼此。
- **相依性聲明**：雖然相依性版本可以從工作區繼承，但每個套件仍需在其 `legion.json` 中顯式列出它所使用的相依項目。
- **相依性提升**：在 `legions.json` 中定義的相依性作為共享池，成員專案透過 `true` 引用。
- **遞迴搜尋**：支援萬用字元模式進行遞迴專案發現

### 2. 相依性管理

- **共享相依性**：在 workspace 階層定義共享的相依性版本
- **版本統一**：確保所有成員專案使用一致的相依性版本
- **相依性解析**：優化相依性解析和建置快取

### 3. 建置協調

- **並行建置**：支援成員專案的並行建置
- **增量建置**：智慧偵測變更，只建置必要的專案
- **建置順序**：根據相依關係自動確定建置順序

### 4. 腳本執行

- **Workspace 腳本**：在 workspace 階層執行的腳本命令
- **批次操作**：對所有成員專案執行相同操作
- **條件執行**：根據專案狀態條件性執行腳本

## 專案間相依性

### 內部相依性

```json5
// project-a/legion.json
{
    "dependencies": {
        "project-b": { "path": "../project-b" },
        "shared-utils": true
    }
}
```

### 相依性解析規則

1. **路徑相依**：使用相對路徑引用其他成員專案
2. **Workspace 相依**：使用 `true` 引用 workspace 階層的相依性
3. **版本約束**：支援版本範圍和精確版本約束

## 開發工作流

### 1. 初始化 Workspace

```bash
# 建立新的 workspace
mkdir my-workspace
cd my-workspace

# 初始化 legions.json
echo '{ "private": true, "members": ["projects/*"] }' > legions.json
```

### 2. 新增成員專案

```bash
# 建立新專案
mkdir projects/my-project
cd projects/my-project

# 初始化專案設定
echo '{ "name": "my-project", "version": "0.1.0" }' > legion.json
```

### 3. 建置和測試

```bash
# 建置所有專案
valkyrie build

# 測試所有專案
valkyrie test

# 建置特定專案
valkyrie build --package my-project

# 執行二進位程式
v tool1                    # 執行 binary/tool1.vk
v tool2                    # 執行 binary/tool2/_.vk
```

## 最佳實踐

### 1. 專案組織

- **邏輯分組**：按功能或層次組織專案
- **清晰命名**：使用一致的專案命名約定
- **文件完整**：為每個專案提供完整的文件

### 2. 相依性管理

- **版本鎖定**：在 workspace 階層鎖定關鍵相依性版本
- **最小相依**：避免不必要的相依性引入
- **定期更新**：定期更新和審查相依性

### 3. 建置優化

- **快取利用**：充分利用建置快取
- **並行建置**：合理配置並行建置參數
- **增量建置**：優化程式碼結構以支援增量建置

## 工具整合

### IDE 支援

- **專案匯入**：IDE 自動識別和匯入 workspace 結構
- **程式碼導覽**：跨專案的程式碼導覽和引用查找
- **除錯支援**：統一的除錯和執行設定

### CI/CD 整合

- **建置矩陣**：支援多專案的建置矩陣
- **測試報告**：聚合的測試結果和覆蓋率報告
- **部署協調**：協調多專案的部署流程

## 遷移指南

### 從單專案到 Workspace

1. **建立 legions.json**：在根目錄建立 workspace 設定
2. **重組專案結構**：將現有程式碼移動到子專案目錄
3. **更新相依性**：調整專案間的相依關係
4. **測試建置**：驗證新結構的建置和測試

### 相容性考慮

- **回溯相容**：保持與現有工具鏈的相容性
- **漸進遷移**：支援漸進式的專案遷移
- **工具適配**：確保開發工具正確識別新結構
