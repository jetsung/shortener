---
name: shortener
description: 对短网址服务（本仓库 shortener-server）中的短链做增删改查——新增、列表查询、查看详情、更新、删除、批量删除。通过 HTTP API 操作一个已运行的服务实例。当用户说"创建/添加/查/改/更新/删除/批量删除 短网址（短址/短链/shorten）"，或要对短码做增删改查时使用。
---

通过 `scripts/shorten.sh`（curl 封装）调用短址 API，对运行中的服务实例完成短链增删改查。

## 前置条件（每次执行前必做）

`SHORTENER_URL` 和凭据都**没有默认值**，不得自行假定服务地址或凭据。必须先确认当前环境中是否已存在，并通过请求验证可用。

**1. 确认变量是否已存在**

```bash
[[ -n "${SHORTENER_URL:-}" ]] && echo "BASE_URL 已设置：${SHORTENER_URL}" || echo "BASE_URL 未设置"
[[ -n "${SHORTENER_KEY:-}" ]] && echo "SHORTENER_KEY 已设置" || echo "SHORTENER_KEY 未设置"
```

- `SHORTENER_URL` 缺失 → 向用户询问服务地址，不要假设 `http://127.0.0.1:8080`。
- `SHORTENER_KEY`（服务端 `server.api_key`，对应 `X-API-KEY` 头）缺失 → 向用户索要。仅支持 API Key 这一种鉴权方式，不接受 JWT / Bearer Token。

**2. 校验地址连通**（`/ping` 无需鉴权）

```bash
curl -sS -m 5 "${SHORTENER_URL%/}/ping"     # 期望返回 {"message":"pong"}
```

**3. 校验凭据可用**（所有 `/api/shortens*` 接口都需鉴权，缺凭据或凭据无效返回 401）

```bash
curl -sS -o /dev/null -w '%{http_code}\n' -m 5 \
  -H "X-API-KEY: ${SHORTENER_KEY}" "${SHORTENER_URL%/}/api/shortens?per_page=1"
```

判定：

| 结果 | 含义 | 处理 |
| --- | --- | --- |
| `200` | 地址与凭据均可用 | 继续执行下面的命令 |
| `401` | 地址可达、凭据无效或缺失 | 向用户索取有效凭据后重试 |
| `000` / curl 连接错误 | 地址不可达 | 向用户确认服务地址，或先启动服务 |

**只有以上校验全部通过，才执行增删改查。** 校验失败时不要继续调用脚本。

## 用法

脚本路径：`scripts/shorten.sh`（相对本 skill 目录）。

```bash
export SHORTENER_URL="<服务地址>"   # 必填，无默认值
export SHORTENER_KEY="<api-key>"         # 必填，唯一鉴权方式

scripts/shorten.sh create <original_url> [short_code] [description]
scripts/shorten.sh list [--page N] [--per-page N] [--sort-by F] [--order asc|desc] [--status N] [--short-code X] [--original-url X]
scripts/shorten.sh get <short_code>
scripts/shorten.sh update <short_code> [--original-url URL] [--description TEXT] [--status N]
scripts/shorten.sh delete <short_code>
scripts/shorten.sh batch-delete <id> [id ...]
scripts/shorten.sh --help
```

成功时输出响应 JSON（删除类为 204，无响应体）；HTTP 状态码不符合预期时以非 0 退出并打印错误体。

## 参数约束

| 项 | 约束 |
| --- | --- |
| `short_code` | 3–16 位、仅字母数字 `^[a-zA-Z0-9]+$`；省略时服务端按 `slug.length`（默认 6）自动生成 |
| `original_url` | 合法 URL，非法返回 400 |
| `description` | 最长 255 字符 |
| `status` | 0=启用，1=禁用，2=未知；禁用后 `GET /{short_code}` 跳转返回 404 |
| 列表分页 | `per_page` 1–100（默认 10），`page` 从 1 开始 |
| `sort_by` | `id` / `short_code` / `created_at` / `updated_at` |
| 搜索 | `original_url` 为模糊匹配；`short_code` 为搜索词 |

## 示例

```bash
# 新增（自动生成短码）
scripts/shorten.sh create "https://example.com/very/long/url" "" "示例"

# 新增（指定短码）：create <url> <short_code> [description]
scripts/shorten.sh create "https://example.com/very/long/url" abc123 "示例"

# 查询：第 1 页、每页 20 条、按创建时间倒序、只查启用状态
scripts/shorten.sh list --page 1 --per-page 20 --sort-by created_at --order desc --status 0

# 模糊搜索原始 URL
scripts/shorten.sh list --original-url example

# 查看 / 更新 / 删除
scripts/shorten.sh get abc123
scripts/shorten.sh update abc123 --original-url "https://example.com/new" --description "新地址"
scripts/shorten.sh update abc123 --status 1          # 禁用
scripts/shorten.sh delete abc123

# 批量删除（按数据库 id，非短码）
scripts/shorten.sh batch-delete 1 2 3
```

## 状态码

| 场景 | 状态码 |
| --- | --- |
| 新增成功 | 201 |
| 查询 / 更新成功 | 200 |
| 删除 / 批量删除成功 | 204 |
| 参数非法、URL 非法、批量删除 ids 为空 | 400 |
| 未鉴权 | 401 |
| 短码不存在 | 404 |
| 短码已存在 | 409 |

## 注意

- 更新为整体字段覆盖：只传需要改的字段，未传的字段不受影响；至少传一个字段。
- 批量删除用的是数据库 `id`（`ShortenResponse.id`），不是 `short_code`；先 `list` 拿到 id 再删。
- 接口契约以仓库根 `openapi.yml` 为准；本 skill 只覆盖 `/api/shortens*` 短址接口，访问历史（`/api/histories*`）不在范围内。
- 短链访问跳转是公开路由 `GET /{short_code}`，无需鉴权，并会异步记录访问历史。
