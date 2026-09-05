# CONTEXT

本文件是本仓库的**领域词汇表**（glossary），不含任何实现细节。

## 术语

- **短链（Short Link）**：由 `short_code` 指向一个 `original_url` 的可访问资源。
- **访问记录（History）**：对短链的每一次访问产生的日志（含 GeoIP / UA 解析）。
- **身份提供方（IdP, Identity Provider）**：遵循 OIDC / OAuth2.0 标准的外部认证服务
  （如 Keycloak、Authelia、Okta、Microsoft Entra ID、自建实现）。本服务**不实现** IdP。
- **OIDC 发现（Discovery）**：通过 IdP 的 issuer URL 下的 `.well-known/openid-configuration`
  自动获取授权/令牌/用户信息端点，无需硬编码各端点。
- **主体标识（sub, Subject）**：IdP 分配给一个用户的稳定内部唯一标识。
- **白名单（Allowlist）**：配置中列出的、被允许登录本服务的标识集合，可为 email 和/或 sub，
  任一命中即放行；名单非空时不在其中的用户被拒绝登录。
- **会话用户（Session User）**：一次 IdP 登录成功后，本服务在内存中缓存的用户视图
  （含 sub / email / 名称），**不持久化到数据库**，重启或多实例间不共享。
- **JWT（JSON Web Token）**：登录成功后由本服务签发的无状态承载令牌（HS256），
  用于替代原先的内存伪随机 token，使多实例可共享校验。
- **API Key**：服务器级单一密钥（`X-API-KEY`），供机器/脚本程序化访问，与人工 OIDC 登录并存。
- **混合认证（Hybrid Auth）**：请求优先用 Bearer JWT 校验，回退到 `X-API-KEY` 头。
- **密码哈希（Password Hash）**：账号密码登录使用的口令不以明文存储，而以 **Argon2id**
  （PHC 字符串格式，自带盐与参数）存于配置。明文口令不落盘。
- **哈希口令生成（Hash Password）**：通过命令行接收明文口令，输出 PHC 串，供填入配置。
  该能力同时内置于 `shortener-cli` 与 `shortener-server`（server 亦可作为独立子命令运行），
  便于单二进制部署时无需额外携带 CLI。

## 前端测试

- **结构快照测试（Structure Snapshot Test）**：对组件渲染出的 DOM 结构做快照比对，而非像素级视觉回归。
  _Avoid_: 视觉回归测试（visual regression）

## 关键事实（非实现细节，而是领域约束）

- 本服务**没有账号系统 / 用户表**。登录身份完全来自外部 IdP 或配置文件中的单账号，本地不落库。
- 登录模型为**单身份、双通道并存**：
  - **OIDC 通道**：仅白名单（email 和/或 sub）内的 IdP 用户可登录。
  - **密码通道**：配置中的单账号，口令以 Argon2id 哈希存储，与 OIDC 并存。
  - 两者均签发 JWT，走同一套 Bearer 校验。
- 配置文件**不再含明文口令**：`admin.password` 改为 `admin.password_hash`（PHC 串）。
- 明文口令登录已从"明文比对"升级为"Argon2id 校验"，并保留 CLI 生成哈希的能力。
