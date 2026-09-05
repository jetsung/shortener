# ADR 0001: OIDC 与账号密码双通道登录

- 状态：已采纳（Accepted）
- 日期：2026-08-17
- 决策人：jetsung

## 背景（Context）

`shortener` 是一个 Rust + Axum 的 URL 短链接服务。改造前的认证是一个**雏形**：

- 仅单管理员账号，凭据来自配置文件 `[admin]` 的明文 `username` / `password`；
- 登录接口 `POST /api/account/login` 做明文字符串比较，成功后生成**伪随机字符串 + 进程内 `HashMap`** 作为 token（24h 过期，重启/多实例即失效）；
- 另有服务器级单一 `X-API-KEY` 供机器/脚本访问（`HybridAuth` 中间件：Bearer 优先，回退 API Key）；
- 数据库**无 users / accounts 表**，用户概念完全不存在；
- 全仓库无 OIDC / OAuth2.0 / SSO 相关实现。

诉求：支持以 **OIDC / OAuth2.0** 方式登录（对接标准 IdP，如 Keycloak、Authelia、Okta、Entra ID），
同时**保留账号密码登录**，并将原本明文存储的口令改为加密/哈希存储。

## 决策（Decision）

采用**单身份、双通道并存**的登录模型：

### 1. OIDC 通道（人工登录）
- 使用**通用 OIDC**（Authorization Code 流），依赖 IdP 的 discovery 文档
  （`.well-known/openid-configuration`）自动发现各端点，不硬编码。
- 登录成功后，用 IdP 返回的 claims 与配置**白名单**比对：支持 `email` 和/或 `sub`，
  **任一命中即放行**；白名单非空且均不命中则**拒绝登录**。
- **email / name 补充来源**：优先从 id_token 读取；若 id_token 缺失 `email`（如 Authelia
  默认只放 `sub`），则调用 userinfo 端点（Bearer access_token）补充获取后再比对白名单，
  userinfo 失败时降级回退 id_token claims（保留告警日志，不阻断登录主流程）。
- 本服务**没有账号系统 / 用户表**：通过 IdP 认证的用户信息仅在**内存中缓存本次会话**，不落库。
- 路由：`GET /api/oidc/login`（重定向 IdP）+
  `GET /api/oidc/callback`（换 token、比对白名单、签发 JWT、302 带 `?token=` 回前端）。

### 2. 密码通道（人工/脚本友好登录）
- **保留**配置文件中的单账号，但口令不再明文：配置字段由 `admin.password` 改为
  `admin.password_hash`，值为 **Argon2id** 哈希，采用 **PHC 字符串格式**（自带盐与参数）。
- 登录时校验哈希而非明文比较。
- 路由：`POST /api/account/login`（保持原路径，校验逻辑改为 Argon2id verify）。

### 3. 会话与令牌
- 两通道登录成功均签发 **JWT（HS256）**，替代原内存伪 token：
  - 无状态，可在多实例间共享校验；
  - 签名密钥为**固定密钥**，优先从环境变量注入（便于容器部署），保证多实例一致。
- `HybridAuth` 中间件结构不变：Bearer JWT 优先，回退 `X-API-KEY`（机器/脚本访问），二者并存。
- 受保护路由（`/api/shortens*`、`/api/histories*`、`/api/account/logout`、`/api/users/current`）不变。

### 4. 哈希口令生成能力（双二进制）
- 为方便部署，**`shortener-cli` 与 `shortener-server` 均内置 `hash-password` 子命令**：
  - 支持**交互式**（不进 shell 历史，推荐）与**显式传参**两种形态，输出 PHC 串；
  - server 默认行为仍是 `serve`，仅当显式传入 `hash-password` 子命令时才走生成逻辑、**不启动 HTTP 服务**；
  - 这样单二进制部署时无需额外携带 CLI 即可生成配置所需的哈希口令。

### 5. 配置
- 新增 `[oidc]` 段：`issuer`、`client_id`、`client_secret`、`allow_emails`、`allow_subjects`；
  文件配置 + 环境变量覆盖（`client_secret` 等敏感项走环境变量）。
- 回调地址（`redirect_uri`）不配置：由服务根据请求 `Host` 头自动推导为
  `https://<域名>/api/oidc/callback`，IdP 登记的回调地址需与之完全一致。
- `enabled` 时 `allow_emails` 与 `allow_subjects` 至少一项非空，否则启动校验失败。

## 理由（Rationale）

- **通用 OIDC + discovery**：一次实现可对接任意标准 IdP，避免逐家适配（Google/GitHub 特例暂不纳入）。
- **白名单而非多用户**：个人使用场景，只需确认 IdP 身份与预期一致，无需用户表/注册/角色，复杂度最低。
- **不落库、内存缓存会话**：与"无账号系统"一致；避免引入 users 表及迁移成本，符合当前数据模型。
- **JWT 替代内存伪 token**：现有结构已是 Bearer token，改为无状态 JWT 改动小，且解决重启失效、多实例不互通问题。
- **Argon2id + PHC**：密码存储现行标准，且代码库内已有实现（原被 `#[allow(dead_code)]` 禁用），零新依赖；PHC 自带盐与参数，verify 无需额外配置。
- **双通道并存**：OIDC 给人工带来免密体验，密码通道给脚本/CLI/无 IdP 环境兜底，互不干扰。
- **哈希生成内嵌 server**：单二进制部署时免去"还需另一个 CLI 二进制"的负担。

## 取舍与后果（Consequences）

- **正向**：移除明文口令存储这一安全隐患；支持标准 IdP 单点登录；多实例可水平扩展；配置可容器化。
- **负向 / 风险**：
  - 内存缓存的会话用户在重启/多实例间不共享——但因 JWT 本身无状态、每次请求独立校验，**实际不影响可用性**；缓存仅用于本次进程内的用户视图。
  - JWT 固定密钥若泄露，攻击者可伪造令牌——故密钥必须来自环境变量且妥善保管。
  - 白名单在配置中，增删用户需改配置重启（个人场景可接受）。
- **未做**（明确排除，避免范围蔓延）：多用户/注册、角色权限模型、users 表持久化、GitHub 等非标准 OAuth2.0 特例适配。

## 参考

- 领域词汇见仓库根 `CONTEXT.md`。
- 相关代码：`shortener-server/src/account.rs`（原明文比对 + 已禁用的 argon2 实现）、
  `shortener-server/src/config.rs`（`AdminConfig` / `ServerConfig`）、
  `shortener-server/src/middleware/hybrid_auth.rs`、`shortener-server/src/router.rs`。
