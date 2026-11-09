#!/usr/bin/env bash
# 打 code-map 服务包并给出上传命令(REQ-274 hub 产物分发,kind=binary)。
#
# 用法: deploy/package.sh [版本标识]     # 缺省用 git short HEAD
# 产物: dist/code-map-<ver>.tar.gz
#
# 包结构(worker 解压后单层目录自动提升):
#   code-map-<ver>/
#     bin/code-map-backend      # Rust release 二进制(平台绑定 x86_64-linux-gnu)
#     frontend/                 # Vite 构建产物(dist)
#     install.env.example       # 运行所需环境变量说明
#
# 平台约束:二进制只能在与构建机同架构/同 libc 的节点运行。
# 跨发行版/ARM 需各自构建并作为不同版本上传(target-triple 区分)。
set -euo pipefail

cd "$(dirname "$0")/.."

VER="${1:-$(git rev-parse --short HEAD)}"
OUT_DIR="dist"
STAGE="$OUT_DIR/code-map-${VER}"
OUT="$OUT_DIR/code-map-${VER}.tar.gz"
HUB="${AGENT_PANEL_URL:-http://192.168.6.133:8788}"
BIN="backend/target/release/code-map-backend"
DIST="frontend/dist"

# 1) 二进制必须存在且足够新(避免打包到陈旧产物)
if [[ ! -x "$BIN" ]]; then
  echo "缺少 $BIN：先构建 —— (cd backend && cargo build --release)" >&2
  exit 1
fi

# 2) 前端产物:缺则用本仓既有脚本构建(npm --prefix frontend run build)
if [[ ! -f "$DIST/index.html" ]]; then
  echo "缺少 $DIST：尝试构建前端…" >&2
  if npm --prefix frontend run build; then
    :
  else
    echo "前端构建失败：先跑 npm run release 后再打包" >&2
    exit 1
  fi
fi

# 3) 组装
rm -rf "$STAGE"
mkdir -p "$STAGE/bin" "$STAGE/frontend"
cp "$BIN" "$STAGE/bin/code-map-backend"
chmod +x "$STAGE/bin/code-map-backend"
cp -r "$DIST/." "$STAGE/frontend/"

cat > "$STAGE/install.env.example" <<'ENVEOF'
# code-map 运行环境变量(服务目录内相对路径)
CODE_MAP_HOST=127.0.0.1
CODE_MAP_PORT=18765
CODE_MAP_FRONTEND_DIR=<安装目录>/frontend
CODE_MAP_DATA_DIR=~/.local/share/code-map
CODE_MAP_ENABLE_WRITE_ACTIONS=true
# 默认扫描的代码工作区根(按需改成本机实际路径)
CODE_MAP_DEFAULT_WORKSPACE=<workspace 根,如 ~/Developer/company/云仓/WMS>
ENVEOF

tar -czf "$OUT" -C "$OUT_DIR" "code-map-${VER}"
rm -rf "$STAGE"

SHA="$(sha256sum "$OUT" | cut -d' ' -f1)"
SIZE="$(stat -c%s "$OUT")"
TARGET="$(uname -m)-$(ldd --version 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+$' || echo unknown)"

cat <<MSG
打包完成
  file   : $OUT
  version: $VER
  target : $TARGET (二进制绑定平台)
  sha256 : $SHA
  size   : $SIZE bytes

上传到 hub:
  # entry/kind 走 query 参数并自行 URL 编码(--data-urlencode 会污染 tar.gz body)
  ENTRY=\$(python3 -c "import urllib.parse;print(urllib.parse.quote('CODE_MAP_FRONTEND_DIR=./frontend ./bin/code-map-backend',safe=''))")
  curl -sS -X PUT "$HUB/api/service/artifact?name=code-map&version=${VER}&kind=binary&entry=\${ENTRY}" \\\\
    -H 'Content-Type: application/gzip' \\\\
    --data-binary @"$OUT"

校验:
  curl -sS "$HUB/api/service/artifact/latest?name=code-map"
MSG
