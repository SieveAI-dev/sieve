# SPEC-005 v2 Wire Fixtures

本目录存放 IPC 协议 v2 的 JSON wire 格式 fixture 文件，每个 method 包含三档：
- `minimal`：仅必填字段
- `full`：全字段（含所有可选字段）
- `null_optional`：可选字段全为 null/默认值

## 目录结构

```
fixtures/v2/
├── README.md               # 本文件
├── sieve.hello/
│   ├── minimal.json
│   ├── full.json
│   └── null_optional.json
├── sieve.set_paused/
│   ├── request.minimal.json
│   └── response.full.json
... (待补全，见 TODO 清单)
```

## TODO（SPEC-005 §14.1 最低门槛：17 method × 3 = 51 条）

- [ ] sieve.hello（notification）
- [ ] sieve.heartbeat（notification）
- [ ] sieve.notify_status_bar（notification）
- [ ] sieve.request_decision（request + response）
- [ ] sieve.request_decision_canceled（notification）
- [ ] sieve.set_paused（request + response）
- [ ] sieve.set_preset（request + response）
- [ ] sieve.set_preset_overrides（request + response）
- [ ] sieve.reload_config（request + response）
- [ ] sieve.health（request + response）
- [ ] sieve.evaluate（request + response）
- [ ] sieve.list_graylist（request + response）
- [ ] sieve.remove_graylist（request + response）
- [ ] sieve.preset_changed（notification）
- [ ] sieve.paused_changed（notification）
- [ ] sieve.reload_user_rules（notification）
- [ ] error responses（parse_error / method_not_found / invalid_params / internal_error）

当前已有 5-10 条示例，见 `schema_v2_fixtures.rs`。
