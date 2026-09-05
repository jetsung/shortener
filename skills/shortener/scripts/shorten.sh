#!/usr/bin/env bash
# 短址 CRUD 命令行封装（基于 shortener HTTP API）
#
# 环境变量：
#   SHORTENER_URL  服务地址（必填，无默认值）
#   SHORTENER_KEY       API Key（X-API-KEY），必填
set -euo pipefail

BASE_URL="${SHORTENER_URL:?未设置 SHORTENER_URL（服务地址，无默认值）}"
API="${BASE_URL%/}/api"

usage() {
  cat <<'EOF'
用法：shorten.sh <命令> [参数]

命令：
  create <original_url> [short_code] [description]   新增短址（short_code 省略则自动生成）
  list   [选项]                                      查询短址列表
  get    <short_code>                                查询单个短址
  update <short_code> [选项]                         更新短址
  delete <short_code>                                删除短址
  batch-delete <id> [id ...]                         批量删除短址

list 选项：
  --page N          页码（默认 1）
  --per-page N      每页条数（默认 10，最大 100）
  --sort-by FIELD   id | short_code | created_at | updated_at（默认 created_at）
  --order DIR       asc | desc（默认 desc）
  --status N        0=启用 1=禁用 2=未知
  --short-code X    短码搜索
  --original-url X  原始 URL 模糊搜索

update 选项：
  --original-url URL   新的原始长网址
  --description TEXT   短链描述
  --status N           0=启用 1=禁用 2=未知

环境变量：
  SHORTENER_URL  服务地址（必填，无默认值）
  SHORTENER_KEY       API Key（必填，唯一鉴权方式）
EOF
}

die() { printf '错误：%s\n' "$1" >&2; exit 1; }

AUTH=()

credentials() {
  [[ -n "${SHORTENER_KEY:-}" ]] || die "缺少凭据：请设置 SHORTENER_KEY（API Key）"
  AUTH=(-H "X-API-KEY: ${SHORTENER_KEY}")
}

# 请求并输出响应；$1 = 期望状态码，其余为 curl 参数
request() {
  local expect="$1"; shift
  local out code
  out="$(curl -sS -w '\n%{http_code}' "$@")"
  code="$(printf '%s' "$out" | tail -n1)"
  out="$(printf '%s' "$out" | sed '$d')"

  if [[ "$code" != "$expect" ]]; then
    printf '请求失败（HTTP %s，期望 %s）\n%s\n' "$code" "$expect" "$out" >&2
    exit 1
  fi
  [[ -z "$out" ]] || printf '%s\n' "$out"
}

cmd="${1:-}"
[[ $# -gt 0 ]] && shift || true

if [[ -z "$cmd" || "$cmd" == "-h" || "$cmd" == "--help" || "$cmd" == "help" ]]; then
  usage
  exit 0
fi

credentials

case "$cmd" in
  create)
    [[ $# -ge 1 ]] || { usage; exit 1; }
    original_url="$1"; short_code="${2:-}"; description="${3:-}"
    body="$(jq -n --arg url "$original_url" '{original_url: $url}')"
    [[ -n "$short_code" ]] && body="$(printf '%s' "$body" | jq --arg c "$short_code" '. + {short_code: $c}')"
    [[ -n "$description" ]] && body="$(printf '%s' "$body" | jq --arg d "$description" '. + {description: $d}')"

    request 201 -X POST "${API}/shortens" \
      -H 'Content-Type: application/json' "${AUTH[@]}" -d "$body"
    ;;

  list)
    page=""; per_page=""; sort_by=""; order=""; status=""; short_code=""; original_url=""
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --page) page="${2:?--page 需要值}"; shift 2 ;;
        --per-page) per_page="${2:?--per-page 需要值}"; shift 2 ;;
        --sort-by) sort_by="${2:?--sort-by 需要值}"; shift 2 ;;
        --order) order="${2:?--order 需要值}"; shift 2 ;;
        --status) status="${2:?--status 需要值}"; shift 2 ;;
        --short-code) short_code="${2:?--short-code 需要值}"; shift 2 ;;
        --original-url) original_url="${2:?--original-url 需要值}"; shift 2 ;;
        *) usage; exit 1 ;;
      esac
    done

    args=()
    [[ -n "$page" ]] && args+=(--data-urlencode "page=$page")
    [[ -n "$per_page" ]] && args+=(--data-urlencode "per_page=$per_page")
    [[ -n "$sort_by" ]] && args+=(--data-urlencode "sort_by=$sort_by")
    [[ -n "$order" ]] && args+=(--data-urlencode "order=$order")
    [[ -n "$status" ]] && args+=(--data-urlencode "status=$status")
    [[ -n "$short_code" ]] && args+=(--data-urlencode "short_code=$short_code")
    [[ -n "$original_url" ]] && args+=(--data-urlencode "original_url=$original_url")

    request 200 -G "${API}/shortens" "${AUTH[@]}" "${args[@]}"
    ;;

  get)
    [[ $# -eq 1 ]] || { usage; exit 1; }
    request 200 "${API}/shortens/$1" "${AUTH[@]}"
    ;;

  update)
    [[ $# -ge 1 ]] || { usage; exit 1; }
    short_code="$1"; shift
    body='{}'
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --original-url) body="$(printf '%s' "$body" | jq --arg v "${2:?--original-url 需要值}" '. + {original_url: $v}')"; shift 2 ;;
        --description) body="$(printf '%s' "$body" | jq --arg v "${2:?--description 需要值}" '. + {description: $v}')"; shift 2 ;;
        --status) body="$(printf '%s' "$body" | jq --argjson v "${2:?--status 需要值}" '. + {status: $v}')"; shift 2 ;;
        *) usage; exit 1 ;;
      esac
    done
    [[ "$body" == '{}' ]] && die "update 至少需要一个字段（--original-url / --description / --status）"

    request 200 -X PUT "${API}/shortens/$short_code" \
      -H 'Content-Type: application/json' "${AUTH[@]}" -d "$body"
    ;;

  delete)
    [[ $# -eq 1 ]] || { usage; exit 1; }
    request 204 -X DELETE "${API}/shortens/$1" "${AUTH[@]}"
    ;;

  batch-delete)
    [[ $# -ge 1 ]] || { usage; exit 1; }
    body="$(jq -n '$ARGS.positional | map(tonumber) | {ids: .}' --args "$@")"

    request 204 -X POST "${API}/shortens/batch-delete" \
      -H 'Content-Type: application/json' "${AUTH[@]}" -d "$body"
    ;;

  *)
    printf '未知命令：%s\n\n' "$cmd" >&2
    usage >&2
    exit 1
    ;;
esac
