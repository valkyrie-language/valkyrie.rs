# PE Parser

直接解析 `PE/COFF/CLI` 二进制结构，不依赖 `ildasm`。

## 范围

1. `DOS` 头、`PE` 签名、`COFF` 头、可选头、数据目录与节头。
2. `CLI` 头与元数据根。
3. `#Strings`、`#US`、`#GUID`、`#Blob`、`#~` 流。
4. 核心元数据表，如 `Module`、`TypeRef`、`TypeDef`、`MethodDef`、`Assembly`、`AssemblyRef`。
