# Actix 到 Tokio 迁移计划

## 已完成

1. **Cargo.toml**
   - 移除: actix, actix-broker, tiny_http, tokio-threadpool
   - 添加: tokio 1.x, axum, tokio-util
   - 升级: libp2p 0.52 → 0.53
   - 添加: kvdb, kvdb-rocksdb
   - edition: 2024 → 2021

2. **lib.rs**
   - 移除 actix, actix_broker 引用
   - 移除已稳定/废弃的 feature

3. **API 模块**
   - 使用 axum 替代 tiny_http

4. **语法修复**
   - trait 对象添加 `dyn`
   - `Vec::remove_item` 改为 `iter().position() + remove()`
   - ValidatorSet 使用 `&dyn ValidatorSet`

## 待完成（核心架构）

### 1. Subscriber 模块 (subscriber/mod.rs, events.rs)

**现状**: 使用 actix 的 ProcessSignals、Recipient、Message

**目标**: 使用 `tokio::sync::broadcast` 或 `tokio::sync::mpsc`

```rust
// 新设计
pub struct P2PEventBus {
    tx: broadcast::Sender<P2PEvent>,
}
impl P2PEventBus {
    pub fn subscribe(&self) -> broadcast::Receiver<P2PEvent> { self.tx.subscribe() }
    pub fn send(&self, event: P2PEvent) { let _ = self.tx.send(event); }
}
```

### 2. Chain (core/chain.rs)

**现状**: 依赖 `Addr<ProcessSignals>`, `Recipient<ChainEvent>`, `post_event`, `subscriber_event`

**目标**: 使用 `broadcast::Sender<ChainEvent>` 替代

### 3. Cmd 主流程 (cmd/mod.rs)

**现状**: `System::run()`, `Addr<>`, `Arbiter::spawn`, `Minner::create`, `TcpServer`, `DiscoverService`

**目标**: `tokio::runtime::Runtime::block_on()` 或 `#[tokio::main]`

### 4. P2P 模块 (p2p/server.rs, session.rs)

**现状**: actix Actor, `FramedWrite`, `subscribe_async`, `do_send`

**目标**: tokio tasks + `tokio_util::codec::Framed`

### 5. DiscoverService (p2p/discover_service.rs)

**现状**: libp2p 0.52 的 `MdnsService`, `MdnsPacket` (0.53 已变更 API)

**目标**: 使用 libp2p 0.53 的 mdns 新 API

### 6. Consensus Core (consensus/pbft/core/core.rs)

**现状**: actix Actor, Handler, Addr, Timer

**目标**: tokio task + channels

### 7. Minner (minner/mod.rs)

**现状**: actix Actor, `subscribe_async`, `BrokerIssue`, `tokio_threadpool`

**目标**: tokio task + `tokio::task::spawn_blocking` 或 thread pool

### 8. BroadcastEventSubscriber / ChainEventSubscriber (subscriber/events.rs)

**现状**: actix_broker::BrokerIssue, BrokerSubscribe

**目标**: tokio broadcast channel

## 依赖变更说明

- **libp2p 0.53**: `MdnsService`/`MdnsPacket` API 已变，需查新文档
- **libp2p 0.53**: `floodsub`, `mplex`, `secio` 可能已移动或重命名
- **crossbeam**: `Sender`/`Receiver` 在 `crossbeam::channel`
- **futures**: `futures::sync::oneshot` → `futures::channel::oneshot`
- **tokio**: `tokio::timer::Delay` → `tokio::time::sleep`
- **tokio**: `tokio::codec` → `tokio_util::codec`

## 建议执行顺序

1. ✅ 实现新的 subscriber (broadcast channel)
2. ✅ 修改 Chain 使用新 subscriber
3. ✅ 修改 cmd 主流程，用 tokio runtime 替代 System::run
4. ✅ 迁移 Minner 为 tokio task (start_minner)
5. ✅ 迁移 Consensus Core 为 tokio task (CoreState, Core::run)
6. ⏳ 迁移 P2P Server/Session (待完成)
7. ⏳ 更新 DiscoverService 以适配 libp2p 0.53 (待完成)

## 当前状态 (2025-03)

- **已完成**: Subscriber, Chain, Cmd, Minner, Consensus Core 已迁移到 tokio
- **cmd**: P2P 和 DiscoverService 暂时注释，节点可启动共识和挖矿
- **待完成**: P2P server/session 需从 actix 迁移到 tokio；DiscoverService 需适配 libp2p 0.53 API
