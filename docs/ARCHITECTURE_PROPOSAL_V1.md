# OpenQBot Architecture Proposal v1

Date: 2026-04-06
Status: In Progress
Scope: architecture proposal for the multi-exchange live trading bot (OpenQBot).

## 1. Goals

- Keep the bot modular and exchange-agnostic.
- Support low-resource environments (Raspberry Pi class hardware).
- Make strategy logic independent of exchange payload formats.
- Add risk and execution boundaries before scaling to more strategies.

## 2. Architectural Principles

- Hexagonal-ish boundaries: domain core in the center, exchange adapters at the edge.
- Typed events over raw JSON across internal layers.
- Strategies generate intents (signals), execution layer places orders.
- Fail-safe runtime: no unwrap/expect in production paths.
- Structured observability: logs, metrics, and explicit lifecycle states.

## 3. Target Module Layout

```text
src/
  main.rs # Entry point, load configuration, dependency wiring
  domain/ # General enums, structs, shared contracts
  strategy/ # Strategies uses market information/indicators to generate signals
  indicators/ # Indicators rely on market stream to extract useful information
  risk/ # Manages position size and risk control
  execution/ # Place Orders, PNL management
  broker/ # Exchange Related (Stream Providers, Place Orders)
  backtest/ # Test Strategies on Historical Data
```

## 4. Core Data Contracts

### 4.1 Market Events

```rust
pub enum MarketEvent {
    Kline(KlineEvent),
    Trade(TradeEvent),
    AggregateTrade(AggregateTradeEvent),
    AveragePrice(AveragePriceEvent),
}

pub struct KlineEvent {
    pub exchange: ExchangeName,
    pub symbol: Symbol,
    pub interval: Interval,
    pub open_time: i64,
    pub close_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub is_closed: bool,
}

pub enum ExchangeName {
  Binance,
  ByBit,
  Kraken,
  ...
}
```

### 4.2 Strategy Output

```rust
pub enum Signal {
    EnterLong { symbol: Symbol, probability: f64 },
    ExitLong  { symbol: Symbol, probability: f64 },
    EnterShort { symbol: Symbol, probability: f64 },
    ExitShort  { symbol: Symbol, probability: f64 },
    Hold,
}
```

### 4.3 Execution Intent

```rust
pub enum TimeInForce {
    FillOrKill,
    ImmediateOrCancel,
    GoodTilCanceled,
    GoodTilDate,
    ...
}

pub struct ExecutionIntent {
    pub strategy_id: String,
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub timestamp: i64,
    pub quantity: Option<f64>,
    pub quote_value: Option<f64>,
    pub limit_price: Option<f64>,
    pub stop_price: Option<f64>,
    pub tif: Option<OrderFillType>,
    pub expire_time: Option<i64>
}
```
