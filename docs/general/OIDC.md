# OIDC / OAuth2.0 登录对接部署指南

本指南说明如何将 Shortener 对接任意标准 OIDC / OAuth2.0 身份提供方（IdP），
实现单点登录（SSO）。配套配置项见 [配置指南 · OIDC 配置](CONFIGURATION.md#oidc-配置)，
设计决策见 [ADR 0001](../adr/0001-oidc-and-password-login.md)。

## 概述

Shortener 支持**两条并存**的登录通道：

1. **OIDC 通道**：对接外部 IdP（Keycloak、Authelia、Okta、Entra ID、Google 等），走标准授权码流。
2. **密码通道**：配置文件中的单管理员账号（Argon2id 哈希口令）。

两条通道均签发无状态 **JWT（HS256）**，前端统一以 `Bearer` 令牌使用，互不影响。

OIDC 登录采用**单身份 + 白名单**模型：

- 本服务**没有账号系统 / 用户表**，用户身份完全来自 IdP，不落库。
- 仅 `allow_emails` / `allow_subjects` 白名单内的 IdP 用户可登录（任一命中即放行）。
- 启用 OIDC（`enabled = true`）时，两项白名单**至少配置一项**，不能都为空（启动校验会拒绝）。

## 前置条件

- 一个可用的 OIDC IdP，且你能在其上创建 OAuth2 / OIDC 客户端。
- 已部署 Shortener 服务（服务端 v0.2.0+，包含 OIDC 与 JWT 支持）。
- 用于签发 JWT 的 `JWT_SECRET`（所有实例必须一致）。

## 步骤一：准备 JWT 签名密钥

两条登录通道签发的 JWT 都用 `JWT_SECRET` 签名。请在所有实例使用**同一个值**，否则多实例间令牌互不相认。

```bash
export JWT_SECRET="$(openssl rand -base64 48)"
```

也可以把密钥写入文件、再用 `JWT_SECRET_FILE` 指向它，便于对接 systemd `LoadCredential`、Docker/K8s Secret 等以文件形式挂载密钥的机制：

```bash
openssl rand -base64 48 > /etc/shortener/jwt_secret
chmod 600 /etc/shortener/jwt_secret
export JWT_SECRET_FILE=/etc/shortener/jwt_secret
```

两者同时设置时以 `JWT_SECRET` 为准；文件内容末尾的换行会被自动去掉。

> 建议通过环境变量 / Secret 管理注入，不要硬编码进配置文件。

## 步骤二：在 IdP 侧创建客户端

以通用 OIDC IdP 为例：

1. 新建一个 **OAuth2 / OIDC 客户端**（应用类型选「Web / 公开或机密均可」）。
2. 授权方式选择 **Authorization Code（授权码流）**，`response_type=code`。
3. 配置 **Redirect URI / Callback URL**，填写本服务的回调地址：

   ```
   https://<你的域名>/api/oidc/callback
   ```

4. 申请 Scope：`openid`、`profile`、`email`。
5. 若 IdP 要求客户端密钥（confidential client），记下 `client_secret`。

> 如果使用 Keycloak：在 Realm 下创建 Client，`Valid Redirect URIs` 填上面的回调地址，
> `Web Origins` 填前端域名；在 Clients 的 Credentials 标签页获取 Secret。

## 步骤三：配置 Shortener

编辑配置文件（默认 `config.toml`），新增 `[oidc]` 段：

```toml
[oidc]
# OIDC 登录总开关（false 时登录不可用）
enabled = true

# IdP 的 issuer。discovery 文档位于 <issuer>/.well-known/openid-configuration
# enabled 时必填
issuer = "https://keycloak.example.com/realms/shortener"

# 步骤二中创建的客户端 ID
client_id = "shortener-app"

# 客户端密钥。强烈建议改用环境变量 OIDC__CLIENT_SECRET 注入，勿写入文件
client_secret = ""

# 回调地址无需配置：服务根据请求的 Host 头自动推导为 https://<域名>/api/oidc/callback。
# 只需确保 IdP 中登记的 Redirect URI 与该地址完全一致

# 白名单：email 或 sub 任一命中即放行（至少配置一项，不能都为空）
allow_emails = ["admin@example.com"]
allow_subjects = []
```

环境变量覆盖（敏感项优先）：

```bash
export OIDC__CLIENT_SECRET="your-client-secret"   # 等价于 [oidc] client_secret
```

`enabled` 为 OIDC 登录的总开关：默认 `false`（不启用）；设为 `true` 并配置 `issuer` / `client_id` / 至少一项白名单后启用 OIDC 登录。

## 步骤四：准备密码通道（可选但推荐保留）

即使启用了 OIDC，也建议保留配置中的管理员账号，用于脚本 / 无 IdP 环境的运维登录。
口令必须以 Argon2id 哈希存储，生成方式：

```bash
# 服务端自带子命令（推荐）
shortener-server hash-password --password "your-secure-password"
# 或使用 CLI
shortener-cli hash-password --password "your-secure-password"
# 交互式（不在 shell 历史留痕）
shortener-server hash-password
```

将输出的整行（`$argon2id$...`）填入配置：

```toml
[admin]
username = "admin"
password_hash = "$argon2id$v=19$m=19456,t=2,p=1$...."
```

## 步骤五：启动并验证

启动服务（确保已注入 `JWT_SECRET`）：

```bash
JWT_SECRET="$JWT_SECRET" shortener-server
```

手动验证 OIDC 流程：

```bash
# 1) 触发登录，应 303 重定向到 IdP 授权页
curl -s -i "https://shortener.example.com/api/oidc/login" | grep -i "^location:"

# 2) 用浏览器打开上面的 location 完成 IdP 登录，
#    IdP 会回跳到 /api/oidc/callback?code=...&state=...
#    回调成功后会 302 到前端并附带 ?token=<jwt>
```

前端登录页已内置「**使用 OIDC 登录**」按钮，点击即发起上述流程；
回调后页面自动从 URL 提取 `token` 并写入 `localStorage`。

> **email / name 来自 id_token 或 userinfo**：服务优先读取 id_token 中的 claims；
> 若 id_token 不含 `email`（部分 IdP 如 Authelia 默认只在 id_token 放 `sub`，
> 其余 claims 需通过 userinfo 端点获取），服务会自动调用 userinfo 端点补充获取
> `email` / `name` 后再进行白名单匹配。userinfo 失败时降级回退 id_token claims。

## 白名单行为说明

| 场景 | 结果 |
|------|------|
| 用户 email 在 `allow_emails` 中 | 放行，签发 JWT |
| 用户 sub 在 `allow_subjects` 中 | 放行，签发 JWT |
| 均不在白名单 | `403 Forbidden`：`User is not in the OIDC allowlist` |
| `allow_emails` 与 `allow_subjects` 均为空 | 配置校验失败，服务无法启动（enabled 时必须至少配置一项） |

> 增删白名单用户需修改配置并**重启服务**。

## 多实例 / 容器部署

- JWT 为无状态 HS256，只要所有实例共享同一 `JWT_SECRET`，即可横向扩展、共享校验。
- `client_secret` 与 `JWT_SECRET` 建议通过容器 Secret / 环境变量注入，而非写进镜像或配置文件。
  以文件形式挂载的 Secret（Docker / K8s）可用 `JWT_SECRET_FILE` 指向挂载路径。
- 配合反向代理（Nginx / Caddy）时，需将 `https://<域名>/api/oidc/callback` 正确转发到本服务。

## 排错

- **未启用 OIDC（`enabled` 未设或为 `false`）**：访问 `/api/oidc/login` 会返回 404（"OIDC login is not enabled"）。
- **`issuer` 配置错误 / 网络不通**：`enabled` 已设为 `true` 但首次访问 `/api/oidc/login` 仍返回 500，
  检查 `issuer` 是否可从本服务访问、`.well-known/openid-configuration` 是否可达。
- **回调 404 / 不匹配**：确认 IdP 登记的 Redirect URI 与本服务自动推导的回调地址（`https://<域名>/api/oidc/callback`）**完全一致**（含协议、域名、路径）。
- **登录后 403**：说明用户不在白名单，检查 `allow_emails` / `allow_subjects`。
  - 白名单匹配用的是 **userinfo 补充后的 `email`**（若 id_token 缺失）：先在 IdP 侧确认该用户的 email 地址，
    再与 `allow_emails` 中的条目逐一比对（大小写不敏感）。
  - Authelia 等 IdP 默认不会把 `email` 放进 id_token，因此即使你在 Authelia 里配置了邮箱，
    也必须确保 IdP 的 userinfo 端点能返回 `email`（通常由 IdP 管理端控制，无需额外配置）。
- **JWT 校验失败（多实例）**：确认各实例 `JWT_SECRET` 一致。
- **启动报 `JWT_SECRET or JWT_SECRET_FILE environment variable is not set`**：`JWT_SECRET` 与 `JWT_SECRET_FILE` 均未注入，请设置其一后重试。
- **启动报 `Failed to read JWT secret file ...`**：`JWT_SECRET_FILE` 指向的文件不存在或当前用户无读取权限，检查路径与权限。

## 安全建议

- 生产环境必须设置 `allow_emails` / `allow_subjects`（至少一项），服务在 `enabled` 且两项均为空时会拒绝启动。
- `client_secret` 与 `JWT_SECRET` 通过环境变量 / Secret 管理，避免入库与进版本控制。
- 配置中的 `admin.password_hash` 与 `oidc.client_secret` 均属于敏感信息，勿提交到代码仓库。
