# Continuous Clearing Auction (CCA) on ICP

**Language:** Rust

**Reference:** [Uniswap CCA](https://docs.uniswap.org/contracts/liquidity-launchpad/CCA)

The **ICP Continuous Clearing Auction (CCA)** is a high-precision, continuous-time token distribution mechanism designed for the Internet Computer. Unlike EVM-based implementations that rely on discrete blocks and tick-based approximations, this implementation leverages the computational capacity of ICP to achieve **millisecond-level precision** and **O(1) settlement complexity** using an accumulator-based mathematical model.

## 1. Overview & Key Advantages

This CCA implementation solves the liquidity bootstrapping problem by allowing the market to determine the fair price of a token over a continuous period.

### Why this architecture is superior:
*   **True Continuous Time:** State updates occur based on millisecond timestamps (`u64`), not discrete block numbers.
*   **O(1) Complexity:** Uses a global accumulator (integral of price inverse) to calculate token fills. Claiming tokens requires constant time regardless of the number of auction steps or duration.
*   **Reactive Outbidding:** Utilizes a Min-Heap to efficiently evict bidders whose `max_price` falls below the current clearing price, ensuring the auction always reflects valid demand.
*   **Dynamic Supply Rate:** The system automatically recalibrates the supply release rate if tokens are under-sold, ensuring the target supply is distributed efficiently by the end of the auction.

## 2. Mathematical Model

The core logic replaces the discrete "Tick" system of Uniswap with a continuous flow model.

### 2.1 Variables
- $S_{total}$: Total Supply to distribute.
- $T_{start}, T_{end}$: Auction duration.
- $F(t)$: Current Global Flow Rate (Total Currency / ns) from all active bidders.
- $R_{supply}(t)$: Supply Rate (Tokens / ns).
- $P(t)$: Clearing Price at time $t$.

### 2.2 Clearing Price & Flow
The clearing price is derived dynamically from the ratio of Demand Flow to Supply Rate:

$$ P(t) = \frac{F(t)}{R_{supply}(t)} $$

Where the Supply Rate is linear but adaptive:
$$ R_{supply}(t) = \frac{\text{Remaining Supply}}{T_{end} - t} $$

### 2.3 Token Allocation (The Accumulator)
To determine how many tokens a bidder receives, we integrate their contribution over time. Since a user's flow rate $f_u$ is constant while they are active, their tokens received ($Q_u$) is:

$$ Q_u = \int_{t_{in}}^{t_{out}} \frac{f_u}{P(t)} dt = f_u \times \int_{t_{in}}^{t_{out}} \frac{1}{P(t)} dt $$

We define the Global Accumulator $\Sigma$:
$$ \Sigma(t) = \int_{0}^{t} \frac{1}{P(\tau)} d\tau $$

Therefore, a user's filled amount is simply:
$$ Q_u = f_u \times (\Sigma(t_{out}) - \Sigma(t_{in})) $$

*This allows for extremely gas-efficient (cycle-efficient) calculations without iterating through history.*

## 3. Lifecycle & Logic

### 3.1 Configuration (`AuctionConfig`)
The auction is initialized with strict parameters to prevent manipulation:
*   `min_bid_duration`: Prevents last-second sniping by enforcing a minimum time exposure for capital.
*   `required_currency_raised`: The "Graduation Threshold". If not met, the auction fails and refunds are enabled.
*   `floor_price`: Implicitly defined by `required_currency_raised` / `total_supply`.

### 3.2 Bidding (`submit_bid`)
When a user submits a bid with Amount $A$ and Max Price $P_{max}$:
1.  **Flow Calculation:** The bid is converted into a flow rate: $f_u = \frac{A}{T_{remaining}}$.
2.  **State Update:** The global state (accumulators, total flow) is updated to the current nanosecond.
3.  **Price Check:** The system verifies $P_{max}$ against the current market.
4.  **Heap Insertion:** The bid is added to the `outbid_heap` (Min-Heap) sorted by price (lowest first).

### 3.3 Outbidding Logic
Every time the state updates or a new bid increases the Global Flow $F(t)$, the Clearing Price $P(t)$ rises.
The system checks the bottom of the heap:
*   If $P_{clearing} > \text{Bidder}_{min}.P_{max}$:
    1.  The bidder is **evicted**.
    2.  Their flow $f_u$ is removed from $F(t)$.
    3.  A snapshot of the accumulator is taken to fix their earnings up to that moment.
    4.  The unspent portion of their currency is marked for refund.

### 3.4 Settlement & Claiming (`claim`)
After the auction ends (or if a user was outbid):
*   **If Graduated:**
    *   `Tokens = FlowRate * (Acc_End - Acc_Start)`
    *   `Refund = OriginalAmount - (FlowRate * DurationActive)` (Refunds "dust" or unspent funds).
*   **If Not Graduated:**
    *   `Tokens = 0`
    *   `Refund = OriginalAmount`

## 4. Data Structures

### `Auction` (Main State)
Maintains the global variables including the "Accumulator per Share" (`acc_tokens_per_share`) which represents the integral of $1/P(t)$.

```rust
pub struct Auction {
    // ... config ...
    supply_rate: u128,          // R_supply
    current_flow_rate: u128,    // F(t)
    acc_tokens_per_share: u128, // The Global Accumulator (Sigma)
    outbid_heap: BinaryHeap<BidOrder>, // For O(log N) access to lowest bids
    // ... stats ...
}
```

### `Bid` (User State)
Stores snapshots to calculate deltas.

```rust
pub struct Bid {
    amount: u128,
    max_price: u128,
    flow_rate: u128,       // User's contribution to pressure
    acc_snapshot: u128,    // Value of Sigma when bid entered
    outbid_time: Option<u64>, // When they were kicked out (if ever)
    // ...
}
```

## 5. Usage Example

### Initialization
```rust
let config = AuctionConfig {
    start_time: 1767225600000,
    end_time: 1767484800000, // 3 days later
    min_bid_duration: 300_000, // 5 minutes
    token_decimals: 9,
    total_supply: 100_000_000_000_000_000, // 100M tokens
    min_amount: 100_000_000, // Min bid
    max_amount: 10_000_000_000,
    required_currency_raised: 500_000_000_000,
};
let mut auction = Auction::new(config);
```

### Submitting a Bid
```rust
// User bids 500 USDC with a max price limit
let bid_info = auction.submit_bid(
    &mut bid_storage,
    500_000_000, // amount
    2_000_000,   // max_price (e.g., 2.0 USDC/Token)
    current_timestamp_ms
)?;
```

### Checkpointing (View)
Frontends can query `get_info()` to show the real-time clearing price and curve.

### Claiming
```rust
// Call after auction ends or after being outbid
let result = auction.claim(&mut bid_storage, bid_id, current_timestamp_ms)?;
println!("Tokens: {}, Refund: {}", result.tokens_filled, result.refund);
```

---

## Conclusion

This Rust implementation for ICP provides a robust, mathematically sound, and computationally efficient mechanism for fair price discovery. By utilizing continuous integral calculus rather than discrete approximation, it eliminates the precision loss and high gas costs associated with EVM-based auction implementations.