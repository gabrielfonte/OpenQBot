# Kraken Integration

## Overview

Kraken provides multiple API options for market data and trading. This document describes the integration strategy using WebSocket v2, chosen for its balance of latency, implementation complexity, and alignment with existing broker integrations.

### API Selection Rationale

Kraken offers three main API approaches:

| API | Latency | Complexity | Rationale |
|-----|---------|-----------|-----------|
| **REST API** | High | Low | Multiple round-trips per subscription; unsuitable for low-latency trading |
| **WebSocket v2** | Low | Medium | Persistent connection, familiar pattern, aligns with Binance implementation |
| **FIX API** | Low | High | Institutional-grade but requires learning curve; WebSocket v2 sufficient for current needs |

**Decision:** WebSocket v2 offers the best combination of low latency and reasonable implementation effort, with a clear upgrade path if needed.

---

## WebSocket API v2 Architecture

### Endpoint Configuration

Kraken WebSocket v2 uses a single endpoint for all market data:

```
wss://ws.kraken.com/v2
```

Authentication (if required for private data) uses bearer tokens via connection parameters.

### Available Market Data Streams

| Stream | Channel | Endpoint | Frequency | Purpose |
|--------|---------|----------|-----------|---------|
| **OHLC** | `ohlc` | `wss://ws.kraken.com/v2` | Configurable (1m, 5m, 15m, etc.) | Kline/Candlestick data |
| **Ticker** | `ticker` | `wss://ws.kraken.com/v2` | Real-time | Level 1 market data (bid/ask, last price) |
| **Book** | `book` | `wss://ws.kraken.com/v2` | Real-time | Level 2 order book snapshots |
| **Orders** | `level3` | `wss://ws-l3.kraken.com/v2` | Real-time | Level 3 individual orders (institutional) |

---

## Authentication & Connection Setup

### Connection Phases

1. **Establish WebSocket connection** to the appropriate endpoint
2. **Receive server status message** (connection acknowledged)
3. **Send subscription requests** to desired channels
4. **Receive confirmation** or error responses

### Message: Connection Status Request

```json
{
  "method": "subscribe",
  "params": {
    "channel": "status"
  }
}
```

**Response:**
```json
{
    "channel": "status",
    "data": [
        {
            "api_version": "v2",
            "connection_id": 13834774380200032777,
            "system": "online",
            "version": "2.0.0"
        }
    ],
    "type": "update"
}
```

### API Key Authentication (for private data)

If accessing private feeds (e.g., account state):
- Include `token` parameter in subscription request
- Token obtained via REST API prior to WebSocket connection
- Token expires after ~15 minutes; implement refresh logic

---

## Message Flow Architecture

### Subscription Model

Subscriptions are product-based. Each subscription request targets one or more products on a specific channel.

### Example: Subscribe to OHLC (1m candles)

```json
{
    "method": "subscribe",
    "params": {
        "channel": "ohlc",
        "symbol": [
            "ALGO/USD",
            "MATIC/USD"
        ],
        "interval": 1
    }
}
```

### Example: OHLC Data Message

```json
{
    "channel": "ohlc",
    "type": "update",
    "timestamp": "2023-10-04T16:26:30.524394914Z",
    "data": [
        {
            "symbol": "MATIC/USD",
            "open": 0.5624,
            "high": 0.5628,
            "low": 0.5622,
            "close": 0.5627,
            "trades": 12,
            "volume": 30927.68066226,
            "vwap": 0.5626,
            "interval_begin": "2023-10-04T16:25:00.000000000Z",
            "interval": 5,
            "timestamp": "2023-10-04T16:30:00.000000Z"
        }
    ]
}
```

### Mapping Strategy: Kraken → EventAndSymbol

| Kraken Stream | EventAndSymbol | Mapping Logic |
|---------------|---|---|
| OHLC candle | `EventAndSymbol::KLine(symbol, interval)` | `data[*].symbol` → symbol; `data[*].interval` → interval |
| Ticker last trade | `EventAndSymbol::Trade(symbol)` | Extract last trade price from ticker data |
| Ticker aggregate | `EventAndSymbol::AggregateTrade(symbol)` | Sum ticker volume into aggregate |
| Ticker average price | `EventAndSymbol::AveragePrice(symbol)` | Compute VWAP from ticker snapshot |

---




