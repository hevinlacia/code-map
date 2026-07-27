---
name: code-map-query
description: 在大型或多 Git 仓库代码库里，先查 code-map 拿到 repo:path:line 候选和关系（谁消费 topic、谁读写表），再精准 read，减少盲搜 token。触发词：代码在哪、谁消费、谁调用、影响面、跨仓定位、ShipmentUploadMq、topic 在哪发、表谁在写。
allowed-tools: ["bash", "read"]
---

# Code Map Query

用于：agent 探索不熟悉或跨多个 Git 仓库的代码库时，先用本地 code-map 服务拿到高置信 `repo:path:line` 候选和关系边，再精准 `read`，避免全仓 `rg`/`read` 把大量无关源码塞进上下文。

适用：
- 跨仓定位：某个类/接口/MQ topic/表/Feign 客户端在哪个 repo、哪一行。
- 影响面：谁消费这个 topic、谁写这张表、谁调这个 Feign。
- 不确定代码在哪个仓库时，先查地图再进具体 repo。

不适用：
- 已知精确文件路径的单文件改动，直接 `read` 更快。
- 非代码仓库（纯文档/数据目录）。
- 需要精确 AST 语义（code-map 是 heuristic，见末尾限制）。

## Trigger

- 用户问“xxx 在哪实现”、“谁消费这个 topic”、“改了 xxx 影响什么”、“这个表谁在写”。
- agent 即将对一个不熟悉的仓库发起宽泛 `rg`/`find` 探索前。
- 跨仓任务：用户在 workspace 根（如 WMS）打开会话，改动涉及多个 repo。

## 前置：确认服务可用

code-map 是本地 systemd 用户服务，默认 `http://127.0.0.1:18765`。

```bash
curl -fsS http://127.0.0.1:18765/health || systemctl --user status code-map --no-pager
```

若服务未运行，启动：`cd /home/hevin/Developer/tools/code-map && npm run deploy`。

CLI 入口：`/home/hevin/Developer/tools/code-map/scripts/code-map-query.sh`
（等价 `npm --prefix /home/hevin/Developer/tools/code-map run query --`）

## 流程

### 1. 定位候选文件：query

```bash
/home/hevin/Developer/tools/code-map/scripts/code-map-query.sh query <term> --json --max-results 8
```

`<term>` 可以是：类名、接口路径（`/api/projects`）、MQ topic、表名、配置 key、文件名片段。

`--json` 输出紧凑结构（agent 优先用这个，体积比人类输出小一个数量级）：

```json
{"project":"WMS workspace","query":"...","count":8,"results":[
  {"repo":"backend/yl-cwhsea-wms-api","path":"wms-shipping/.../ShipmentUploadMq.groovy",
   "line":61,"score":469,"reason":"repo path contains 'shipmentuploadmq'",
   "symbols":[{"kind":"class","name":"ShipmentUploadMq","line":61},{"kind":"mq_topic","name":"wms-shipment-upload-topic","line":113}]}
]}
```

每个 hit 只含 `repo + path + line + score + reason + symbols`，没有源码片段。拿到后**只读 top 3 的具体行**，不要全仓 grep。

### 2. 影响面 / 关系解析：neighbors

```bash
/home/hevin/Developer/tools/code-map/scripts/code-map-query.sh neighbors <entity> --json
```

`<entity>` 可以是：topic 字符串、表名、Feign 客户端名、类名、常量名。

返回分桶（每条都是 `repo:path:line`）：

- `definitions` - 该实体在哪定义（常量/类/表符号）
- `producers` - 谁发布这个 MQ topic（`mq_publish`）
- `consumers` - 谁消费这个 topic（`mq_consume`）
- `readers` - 谁读这张表（`sql_table_read`）
- `writers` - 谁写这张表（`sql_table_write`）
- `callers` - 谁调这个 Feign/Dubbo/前端 API

一次调用直接拿到影响面，不用读源码。例如 `neighbors shipment_header` 直接返回所有读写该表的 repo:path:line。

### 3. 索引过期时重扫

如果查询结果明显过时（文件已删/改名）或返回空但确定代码存在，重扫：

```bash
# 增量重扫（默认，~0.4s，只读变更文件，复用上次常量表）
BASE=http://127.0.0.1:18765
PID=$(curl -fsS "$BASE/api/projects" | jq -r '.[] | select(.name=="WMS workspace") | .id')
curl -fsS -X POST "$BASE/api/projects/$PID/scan" | jq '{file_count,symbol_count,relationship_count}'

# 常量定义被改动后，用 force=true 全量重扫重建常量表（~4s）
curl -fsS -X POST "$BASE/api/projects/$PID/scan?force=true" | jq '{file_count,symbol_count,relationship_count}'
```

WMS 全量扫描约 4s，增量约 0.4s。扫描只读源码建索引，不修改任何仓库。

### 4. 沉淀确认过的链路（verified notes）

当 agent 确认了一条调用链/影响面后，把它写成 note，下次同类查询直接命中，不再重新搜：

```bash
BASE=http://127.0.0.1:18765
curl -fsS -X POST "$BASE/api/notes" -H 'Content-Type: application/json' -d '{
  "query": "ShipmentUploadMq",
  "summary": "ShipmentUploadMq 处理出库回传；由 ShipmentUploadRocketListener 消费 wms-shipment-upload-topic 触发，读 shipping_container_header/detail。",
  "pointers": [
    {"repo": "backend/yl-cwhsea-wms-api", "path": "wms-shipping/.../ShipmentUploadMq.groovy", "line": 61, "note": "主类"}
  ]
}'
```

之后 `query ShipmentUploadMq --json` 的 `notes` 字段会带上这条结论。note 的 query 与新查询模糊匹配（任一方包含对方）即命中。

## 输出判读

- `repo` 是相对 workspace 根的仓库路径（如 `backend/yl-cwhsea-wms-api`）。
- `path` 是 repo 内相对路径。
- 完整定位 = `<workspace_root>/<repo>/<path>:<line>`。
- `score` 越高越相关；`reason` 说明命中类型（path/symbol/relationship/content）。
- `symbols` 里的 `kind` 常见：`class`、`controller_route`、`mq_topic`、`mq_consumer`、`feign_client`、`db_table`、`frontend_api_call`。

## Required Checks

- 查询前确认服务在线（`/health`）。
- 优先用 `--json`，不用人类输出，省 token。
- `query` 拿到候选后，只 `read` top 几个文件的具体行号附近，不要把整个文件或整仓 grep 结果塞进上下文。
- `neighbors` 的 `producers`/`consumers` 对**常量间接引用**（如 `ShipmentTopicConstants.WMS_SHIPMENT_UPLOAD_TOPIC`）可能为空——此时先看 `definitions` 找到常量定义文件，再去该文件确认发送/消费点。

## 限制（向用户如实说明）

- 索引是 **heuristic 抽取**，不是完整 AST：基于注解、字符串字面量、SQL 关键字的模式匹配。
- **常量间接引用未解析**：发送处用 `Constants.TOPIC` 而非字符串字面量时，`producers`/`consumers` 可能漏；`definitions` 仍能定位常量。
- SQL 表名抽取可能误报自然语言里的 `from`/`join`（如文档里的 "from the"）。
- 索引在扫描时刻冻结；大改后需手动 rescan 才会更新。

## Final Response

向用户报告时给出：
- 查询的 term/entity 和命中的 top `repo:path:line`。
- 如果做了 neighbors，给出各分桶数量和关键命中。
- 明确标注哪些是 code-map heuristic 结果，建议用户/agent 复核的文件。
