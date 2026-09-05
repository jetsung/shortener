# 通用文档

本目录包含 Shortener 项目的通用文档，适用于服务器和 CLI 工具。

## 文档列表

- [安装指南](INSTALLATION.md) - 详细的安装说明和多种安装方式
- [配置说明](CONFIGURATION.md) - 服务器和 CLI 配置选项
- [配置文件说明](CONFIG_FILES.md) - 配置文件与加载机制
- [环境变量参考](ENVIRONMENT_VARIABLES.md) - 完整的环境变量参数说明
- [GeoIP 配置指南](GEOIP.md) - 地理位置追踪配置
- [OIDC 对接部署](OIDC.md) - OIDC / OAuth2.0 单点登录接入指南

## 快速开始

### 安装

查看 [安装指南](INSTALLATION.md) 了解：
- 从源码编译
- 使用预编译二进制
- 使用 Docker
- 使用包管理器

### 配置

查看 [配置说明](CONFIGURATION.md) 了解：
- 服务器配置选项
- CLI 配置选项
- 数据库配置
- 缓存配置
- GeoIP 配置

完整的环境变量清单（含 JWT_SECRET）请参阅 [环境变量参考](ENVIRONMENT_VARIABLES.md)。

### OIDC 登录

查看 [OIDC 对接部署](OIDC.md) 了解：
- IdP 客户端创建
- `JWT_SECRET` 与 `[oidc]` 配置
- 允许列表（邮箱 / sub）
- 密码哈希生成与双通道登录

## 相关文档

- [服务器文档](../server/README.md)
- [CLI 文档](../cli/README.md)
- [部署指南](../deployment/README.md)
