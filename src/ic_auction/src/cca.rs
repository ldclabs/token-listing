//! # Continuous Clearing Auction
//!
//! This module implements a Continuous Clearing Auction (CCA) mechanism for token sales on the Internet Computer (IC).
//!
//! References: https://docs.uniswap.org/contracts/liquidity-launchpad/CCA
//!
use candid::Nat;
use ciborium::{from_reader, into_writer};
use ic_stable_structures::{Storable, storable::Bound};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::{BTreeMap, BinaryHeap},
};

use crate::types::{AuctionConfig, AuctionInfo, AuctionSnapshot, BidInfo};

// Precision constants
const RATE_PRECISION: u128 = 1_000_000_000; // Flow rate precision (1e9)
const ACC_PRECISION: u128 = 1_000_000_000_000_000_000; // Accumulator precision (1e18)

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bid {
    // User's currency amount
    #[serde(rename = "a")]
    pub amount: u128,
    // The max price of the bid
    #[serde(rename = "p")]
    pub max_price: u128,

    // --- Dynamic Settlement Fields ---
    #[serde(rename = "f")]
    pub flow_rate: u128, // Flow rate (Currency / ms)
    #[serde(rename = "s")]
    pub acc_snapshot: u128, // Global accumulator snapshot at entry

    #[serde(rename = "c")]
    pub create_time: u64, // Creation time, allows early entry
    #[serde(rename = "o")]
    pub outbid_time: Option<u64>, // Time when the bid was outbid
    #[serde(rename = "os")]
    pub outbid_acc_snapshot: Option<u128>, // Global accumulator snapshot when outbid

    #[serde(rename = "tf")]
    pub tokens_filled: u128, // Amount of tokens filled
    #[serde(rename = "r")]
    pub refund: u128, // Amount of currency refunded
    #[serde(rename = "ct")]
    pub claim_time: u64, // Claim/Settlement time
}

impl Bid {
    pub fn into_info(self, id: u64) -> BidInfo {
        BidInfo {
            id,
            amount: self.amount,
            max_price: self.max_price,
            flow_rate: self.flow_rate,
            acc_snapshot: self.acc_snapshot,
            create_time: self.create_time,
            outbid_time: self.outbid_time,
            outbid_acc_snapshot: self.outbid_acc_snapshot,
            tokens_filled: self.tokens_filled,
            refund: self.refund,
            claim_time: self.claim_time,
        }
    }
}

impl Storable for Bid {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode Bid data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode Bid data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode Bid data")
    }
}

pub trait BidStorage {
    fn get(&self, bid_id: u64) -> Option<Bid>;
    fn insert(&self, bid_id: u64, bid: Bid);
}

// Helper struct for Min-Heap (lowest price at the top)
#[derive(Clone, Eq, PartialEq, Deserialize, Serialize)]
struct BidOrder {
    #[serde(rename = "i")]
    id: u64,
    #[serde(rename = "p")]
    max_price: u128,
    #[serde(rename = "a")]
    amount: u128,
}

// Implement Ord to reverse order (Min-Heap)
impl Ord for BidOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lower price has higher priority (more likely to be outbid)
        other
            .max_price
            .cmp(&self.max_price)
            .then_with(|| other.amount.cmp(&self.amount))
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for BidOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Main struct: Auction State
#[derive(Clone, Deserialize, Serialize)]
pub struct Auction {
    cfg: AuctionConfig,
    // 10 ** token_decimals
    one_token: u128,
    // Floor price per token, in currency atomic units
    floor_price: u128,

    // Price precision multiplier, determined by floor price
    price_precision: u128, // default to 1

    // --- Global State ---
    next_bid_id: u64,
    last_update_time: u64,

    // Pre-calculated constant: Tokens released per nanosecond (Supply Rate)
    // For linear release: supply_rate = total_supply / duration
    supply_rate: u128,

    // Total currency amount participating in the auction
    total_amount: u128,

    // Total tokens filled
    total_tokens_filled: u128,

    // Total currency refunded
    // total_refunded == total_amount - cumulative_demand_raised
    total_refunded: u128,

    // Cumulative tokens released Q_total(t)
    cumulative_supply_released: u128,
    // Cumulative currency raised D_total(t)
    cumulative_demand_raised: u128,
    // Total flow rate of all active bids
    current_flow_rate: u128,
    // Accumulator: Total tokens obtained per unit of flow rate
    // Integral( 1 / P(t) ) dt
    acc_tokens_per_share: u128,

    // Candidate eviction queue (Min-Heap)
    outbid_heap: BinaryHeap<BidOrder>,
}

// =============================================================================
// 3. Core Logic Implementation
// =============================================================================

impl Auction {
    pub fn new(cfg: AuctionConfig, token_decimals: u8) -> Result<Self, String> {
        let supply_rate = Nat::from(cfg.total_supply) * Nat::from(RATE_PRECISION)
            / Nat::from(cfg.end_time - cfg.start_time);
        let supply_rate: u128 = supply_rate
            .0
            .try_into()
            .map_err(|_| "Supply rate overflow".to_string())?;
        if supply_rate == 0 {
            return Err("Supply rate too low for the auction duration".to_string());
        }
        let one_token = 10u128.pow(token_decimals as u32);
        let floor_price = Nat::from(cfg.required_currency_raised) * Nat::from(one_token)
            / Nat::from(cfg.total_supply);
        let floor_price: u128 = floor_price
            .0
            .try_into()
            .map_err(|_| "Floor price overflow".to_string())?;
        if floor_price == 0 {
            return Err("Floor price too low".to_string());
        }

        // Determine price precision based on floor price
        let price_precision = if floor_price >= 1_000_000_000 {
            1
        } else if floor_price >= 1_000_000 {
            1_000
        } else if floor_price > 1_000 {
            1_000_000
        } else {
            1_000_000_000
        };
        let floor_price = floor_price * price_precision;
        Ok(Self {
            cfg: cfg.clone(),
            one_token,
            floor_price,
            price_precision,
            last_update_time: cfg.start_time,
            supply_rate,
            total_amount: 0,
            total_tokens_filled: 0,
            total_refunded: 0,
            cumulative_supply_released: 0,
            cumulative_demand_raised: 0,
            current_flow_rate: 0,
            acc_tokens_per_share: 0,
            outbid_heap: BinaryHeap::new(),
            next_bid_id: 1,
        })
    }

    pub fn get_info(&self, now_ms: u64) -> AuctionInfo {
        let clearing_price = self.get_clearing_price();
        let mut info = AuctionInfo {
            timestamp: now_ms,
            clearing_price: clearing_price / self.price_precision,
            total_amount: self.total_amount,
            total_tokens_filled: self.total_tokens_filled,
            total_refunded: self.total_refunded,
            cumulative_demand_raised: self.cumulative_demand_raised / self.price_precision,
            cumulative_supply_released: self.cumulative_supply_released,
            is_graduated: self.is_graduated(),
            bids_count: self.next_bid_id - 1,
        };

        if now_ms < self.cfg.end_time && now_ms > self.last_update_time {
            let dt = Nat::from(now_ms - self.last_update_time);

            if clearing_price > 0 {
                let supply_delta = Nat::from(self.current_flow_rate)
                    * dt.clone()
                    * Nat::from(self.one_token * self.price_precision)
                    / (Nat::from(RATE_PRECISION) * Nat::from(clearing_price));
                let supply_delta: u128 = supply_delta.0.try_into().expect("Supply delta overflow");
                info.cumulative_supply_released += supply_delta;
            }

            let demand_delta = Nat::from(self.current_flow_rate) * dt.clone()
                / Nat::from(RATE_PRECISION * self.price_precision);
            let demand_delta: u128 = demand_delta.0.try_into().expect("Demand delta overflow");
            info.cumulative_demand_raised += demand_delta;
        }

        info
    }

    /// Get grouped bids information
    // Return: Vec<(price_range_start, total_amount_in_range)>
    pub fn get_grouped_bids(&self, precision: u128) -> Vec<(u128, u128)> {
        let mut price_buckets: BTreeMap<u128, u128> = BTreeMap::new();

        // Iterate through all active bids
        for bid_order in self.outbid_heap.iter() {
            let price_range_key = (bid_order.max_price / precision) * precision;
            let amount = price_buckets.entry(price_range_key).or_insert(0);
            *amount += bid_order.amount;
        }

        price_buckets.into_iter().collect()
    }

    pub fn is_graduated(&self) -> bool {
        self.cumulative_demand_raised >= self.cfg.required_currency_raised * self.price_precision
    }

    pub fn is_ended(&self, now_ms: u64) -> bool {
        now_ms > self.cfg.end_time
    }

    pub fn currency_raised(&self) -> u128 {
        if self.is_graduated() {
            self.cumulative_demand_raised / self.price_precision
        } else {
            0
        }
    }

    pub fn tokens_sold(&self) -> u128 {
        if self.is_graduated() {
            self.cumulative_supply_released
        } else {
            0
        }
    }

    /// Calculate current "Clearing Price": Currency Atomic Units per One Token
    /// Price = (Current_Flow / Supply_Rate)
    pub fn get_clearing_price(&self) -> u128 {
        if self.supply_rate == 0 {
            return 0;
        }

        // Price = (Flow Rate / Supply Rate)
        // Note: Flow rate includes RATE_PRECISION
        // Price = (Flow / RATE_PRECISION) / (SupplyRate / (one_token * RATE_PRECISION))
        // To round up: (numerator + denominator - 1) / denominator
        let numerator = Nat::from(self.current_flow_rate) * Nat::from(self.one_token);
        let denominator = Nat::from(self.supply_rate);
        let price_nat = (numerator + denominator.clone() - Nat::from(1u64)) / denominator;
        let price: u128 = price_nat.0.try_into().expect("Clearing price overflow");
        self.floor_price.max(price)
    }

    pub fn get_max_price_threshold(&self, flow_rate: u128) -> (u128, u128) {
        if self.supply_rate == 0 {
            return (0, 0);
        }
        let clearing_price = self.get_clearing_price();
        let price_delta = Nat::from(flow_rate) * Nat::from(self.one_token);
        let denominator = Nat::from(self.supply_rate);
        let price_delta = (price_delta + denominator.clone() - Nat::from(1u64)) / denominator;
        let price_delta: u128 = price_delta.0.try_into().expect("Clearing price overflow");
        (clearing_price, clearing_price + price_delta)
    }

    /// State advancement and settlement
    /// Includes: Time advancement -> Token accumulation -> Currency accumulation -> Accumulator update
    pub fn update_state(&mut self, now_ms: u64) {
        if now_ms <= self.last_update_time {
            return;
        }

        // 1. Determine valid time window (Stop updating if auction ended)
        let valid_end = now_ms.min(self.cfg.end_time);
        if valid_end <= self.last_update_time {
            self.last_update_time = now_ms;
            return;
        }

        // ---------------------------------------------------------------------
        // Linear Release & Accumulation
        // ---------------------------------------------------------------------
        let dt = Nat::from(valid_end - self.last_update_time);

        // D(t) Delta: Based on current flow rate
        // Delta D = FlowRate * dt
        let demand_delta =
            Nat::from(self.current_flow_rate) * dt.clone() / Nat::from(RATE_PRECISION);
        let demand_delta: u128 = demand_delta.0.try_into().expect("Demand delta overflow");
        self.cumulative_demand_raised += demand_delta;

        if self.supply_rate > 0 {
            let clearing_price = self.get_clearing_price();

            // Tokens obtained = Flow * dt / Price
            // acc_delta = dt / Price
            let acc_delta = (dt.clone() * Nat::from(ACC_PRECISION)) / Nat::from(clearing_price);
            let acc_delta: u128 = acc_delta.0.try_into().expect("Accumulator delta overflow");
            self.acc_tokens_per_share += acc_delta;

            // Q(t) Delta: Actual sold tokens
            // Delta Q = Flow * dt / Price
            let supply_delta = Nat::from(self.current_flow_rate) * dt * Nat::from(self.one_token)
                / (Nat::from(RATE_PRECISION) * Nat::from(clearing_price));
            let supply_delta: u128 = supply_delta.0.try_into().expect("Supply delta overflow");
            self.cumulative_supply_released += supply_delta;
        }

        // Recalculate Supply Rate based on remaining tokens and time
        // This ensures that if tokens were under-sold (due to floor price),
        // the supply rate increases for the remaining duration to try to sell them all.
        let remaining_time = self.cfg.end_time.saturating_sub(valid_end);
        if remaining_time > 0 {
            let remaining_supply = self
                .cfg
                .total_supply
                .saturating_sub(self.cumulative_supply_released);
            let new_rate =
                Nat::from(remaining_supply) * Nat::from(RATE_PRECISION) / Nat::from(remaining_time);
            self.supply_rate = new_rate.0.try_into().expect("Supply rate overflow");
        }

        self.last_update_time = now_ms;
    }

    /// Handle Outbid logic
    fn process_outbids<B: BidStorage>(&mut self, bids: &B, now_ms: u64) -> u128 {
        loop {
            let clearing_price = self.get_clearing_price();

            let should_pop = match self.outbid_heap.peek() {
                Some(candidate) => candidate.max_price * self.price_precision < clearing_price,
                None => false,
            };

            if !should_pop {
                return clearing_price;
            }

            if let Some(candidate) = self.outbid_heap.pop() {
                self.execute_outbid(bids, candidate.id, now_ms);
            }
        }
    }

    fn execute_outbid<B: BidStorage>(&mut self, bids: &B, bid_id: u64, now_ms: u64) {
        if let Some(mut bid) = bids.get(bid_id) {
            if bid.outbid_time.is_some() {
                return;
            }

            // 1. Remove from global flow rate
            // Means from the next nanosecond, its funds no longer drive demand growth or buy tokens
            self.current_flow_rate = self.current_flow_rate.saturating_sub(bid.flow_rate);

            // 2. Mark state
            bid.outbid_time = Some(now_ms);

            // 3. Snapshot accumulator (Fix earnings)
            bid.outbid_acc_snapshot = Some(self.acc_tokens_per_share);

            // 4. Calculate tokens filled and refund
            let acc_growth = self.acc_tokens_per_share.saturating_sub(bid.acc_snapshot);
            // Tokens = Flow Rate * Accumulator Delta / Precision
            let tokens_filled =
                Nat::from(bid.flow_rate) * Nat::from(acc_growth) * Nat::from(self.one_token)
                    / Nat::from(RATE_PRECISION * ACC_PRECISION);
            bid.tokens_filled = tokens_filled.0.try_into().unwrap_or_default();
            // Refund: "Total - Spent", to refund dust from division
            // Spent = flow_rate * (now - create_time)
            // Using max(start_time) for safety
            let effective_start = bid.create_time.max(self.cfg.start_time);
            let spent_duration = now_ms.saturating_sub(effective_start);
            let spent = Nat::from(bid.flow_rate) * Nat::from(spent_duration)
                / Nat::from(RATE_PRECISION * self.price_precision);
            let spent: u128 = spent.0.try_into().unwrap_or_default();

            bid.refund = bid.amount.saturating_sub(spent);

            bids.insert(bid_id, bid);
        }
    }

    pub fn estimate_max_price(&self, amount: u128, now_ms: u64) -> (u128, u128) {
        let remaining_time = self
            .cfg
            .end_time
            .saturating_sub(now_ms.max(self.cfg.start_time));
        if remaining_time < self.cfg.min_bid_duration {
            return (0, 0);
        }

        // Calculate flow rate: Linear distribution
        let flow_rate = Nat::from(amount) * Nat::from(RATE_PRECISION * self.price_precision)
            / Nat::from(remaining_time);
        let flow_rate: u128 = flow_rate.0.try_into().unwrap_or_default();
        if flow_rate == 0 {
            return (0, 0);
        }
        let (clearing_price, max_price_threshold) = self.get_max_price_threshold(flow_rate);

        (
            clearing_price.div_ceil(self.price_precision),
            max_price_threshold.div_ceil(self.price_precision),
        )
    }

    /// Submit Bid
    pub fn submit_bid<B: BidStorage>(
        &mut self,
        bids: &B,
        amount: u128,
        max_price: u128,
        now_ms: u64,
    ) -> Result<(BidInfo, AuctionSnapshot), String> {
        if now_ms >= self.cfg.end_time {
            return Err("AuctionEnded: Auction is ended".to_string());
        }
        if amount < self.cfg.min_amount {
            return Err("InvalidBidAmount: Bid amount below minimum required".to_string());
        }
        if amount > self.cfg.max_amount {
            return Err("InvalidBidAmount: Bid amount exceeds maximum allowed".to_string());
        }

        let remaining_time = self
            .cfg
            .end_time
            .saturating_sub(now_ms.max(self.cfg.start_time));
        if remaining_time < self.cfg.min_bid_duration {
            return Err(
                "InvalidBidDuration: Not enough time remaining for the minimum bid duration"
                    .to_string(),
            );
        }

        // Calculate flow rate: Linear distribution
        let flow_rate = Nat::from(amount) * Nat::from(RATE_PRECISION * self.price_precision)
            / Nat::from(remaining_time);
        let flow_rate: u128 = flow_rate.0.try_into().unwrap_or_default();

        if flow_rate == 0 {
            return Err(
                "InvalidBidAmount: Bid amount too low for the remaining auction duration"
                    .to_string(),
            );
        }

        // Must update state to latest first
        self.update_state(now_ms);

        let pp = max_price * self.price_precision;
        let (_, mpt) = self.get_max_price_threshold(flow_rate);
        if pp < mpt {
            return Err("InvalidBidPrice: Price limit below current market".to_string());
        }
        if pp >= mpt * 1000 {
            return Err("InvalidBidPrice: Price limit too high".to_string());
        }

        let id = self.next_bid_id;
        self.next_bid_id += 1;

        // Increase global flow rate
        self.current_flow_rate += flow_rate;
        // Increase total amount
        self.total_amount += amount;

        let bid = Bid {
            amount,
            max_price,
            flow_rate,
            acc_snapshot: self.acc_tokens_per_share, // Record entry accumulator
            create_time: now_ms,
            outbid_time: None,
            outbid_acc_snapshot: None,
            tokens_filled: 0,
            refund: 0,
            claim_time: 0,
        };

        bids.insert(id, bid.clone());
        self.outbid_heap.push(BidOrder {
            id,
            max_price,
            amount,
        });

        // Check and evict users with insufficient price (Reactive Outbid)
        let clearing_price = self.process_outbids(bids, now_ms);

        Ok((
            bid.into_info(id),
            AuctionSnapshot {
                timestamp: now_ms,
                clearing_price: clearing_price / self.price_precision,
                current_flow_rate: self.current_flow_rate,
                cumulative_demand_raised: self.cumulative_demand_raised / self.price_precision,
                cumulative_supply_released: self.cumulative_supply_released,
            },
        ))
    }

    /// Claim / Settlement
    /// Returns settled Bid details
    pub fn claim<B: BidStorage>(
        &mut self,
        bids: &B,
        bid_id: u64,
        now_ms: u64,
    ) -> Result<BidInfo, String> {
        let mut bid = bids.get(bid_id).ok_or(format!("BidNotFound: {}", bid_id))?;

        if bid.claim_time > 0 {
            return Err("BidClaimed: Already claimed".to_string());
        }

        self.update_state(now_ms);
        // If already Outbid and auction graduated, settle directly
        let is_graduated = self.is_graduated();
        if is_graduated && bid.outbid_time.is_some() {
            bid.claim_time = now_ms;
            self.total_tokens_filled += bid.tokens_filled;
            self.total_refunded += bid.refund;
            bids.insert(bid_id, bid.clone());

            return Ok(bid.into_info(bid_id));
        }

        if now_ms <= self.cfg.end_time {
            return Err("NotClaimable: Cannot claim yet".to_string());
        }

        if !is_graduated {
            // Auction not graduated, full refund
            bid.tokens_filled = 0;
            bid.refund = bid.amount;
            self.total_refunded += bid.refund;
        } else {
            // 1. Calculate tokens filled
            // Delta = End - Start
            let acc_growth = self.acc_tokens_per_share.saturating_sub(bid.acc_snapshot);
            // Tokens = Flow Rate * Accumulator Delta / Precision
            let tokens_filled =
                Nat::from(bid.flow_rate) * Nat::from(acc_growth) * Nat::from(self.one_token)
                    / Nat::from(RATE_PRECISION * ACC_PRECISION);
            bid.tokens_filled = tokens_filled.0.try_into().unwrap_or_default();

            self.total_tokens_filled += bid.tokens_filled;

            // 2. Calculate remaining dust refund
            // Actual spent = Flow Rate * Duration
            // Duration = end_time - create_time (or start_time)
            // Note: bid.create_time might be earlier than start_time, take max
            let duration = self
                .cfg
                .end_time
                .saturating_sub(bid.create_time.max(self.cfg.start_time));
            let actual_spent = Nat::from(bid.flow_rate) * Nat::from(duration)
                / Nat::from(RATE_PRECISION * self.price_precision);
            let actual_spent: u128 = actual_spent.0.try_into().unwrap_or_default();

            // Refund unspent balance (Dust)
            bid.refund = bid.amount.saturating_sub(actual_spent);
            self.total_refunded += bid.refund;
        }

        bid.claim_time = now_ms;
        bids.insert(bid_id, bid.clone());
        Ok(bid.into_info(bid_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    struct MockBidStorage {
        bids: RefCell<HashMap<u64, Bid>>,
    }

    impl BidStorage for MockBidStorage {
        fn get(&self, bid_id: u64) -> Option<Bid> {
            self.bids.borrow().get(&bid_id).cloned()
        }
        fn insert(&self, bid_id: u64, bid: Bid) {
            self.bids.borrow_mut().insert(bid_id, bid);
        }
    }

    fn get_test_config() -> AuctionConfig {
        AuctionConfig {
            start_time: 1000,
            end_time: 11000, // 10s duration
            min_bid_duration: 100,
            total_supply: 1000 * 100_000_000,   // 1000 tokens
            liquidity_pool_amount: 100_000_000, // 100 tokens
            min_amount: 1000,
            max_amount: 1_000_000_000,
            required_currency_raised: 100_000,
        }
    }

    #[test]
    fn test_basic_auction_flow() {
        let cfg = get_test_config();
        let mut auction = Auction::new(cfg.clone(), 8).unwrap();
        let storage = MockBidStorage::default();
        let price_precision = auction.price_precision; // 1e9

        // 1. Submit Bid before auction start
        let (bid1, _) = auction.submit_bid(&storage, 50_000, 500, 1).unwrap();
        // Flow rate 1 = 50000 * 1e9 / 10000 = 5 * 1e9
        assert_eq!(bid1.flow_rate, 5 * RATE_PRECISION * price_precision);
        assert_eq!(auction.current_flow_rate, bid1.flow_rate);
        assert_eq!(auction.cumulative_supply_released, 0);
        // supply rate = 1000 tokens * 1e8 * RATE_PRECISION / 10000 ms = 0.1 tokens/ms
        assert_eq!(auction.supply_rate, 10_000_000 * RATE_PRECISION);
        // Price =  global_flow_rate * one token / supply_rate =
        // (5 * 1e9 * 1e8) / (10_000_000 * 1e9) = 50
        assert_eq!(auction.get_clearing_price(), 100 * price_precision); // Floor price

        let err = auction.claim(&storage, bid1.id, 2000).err().unwrap();
        assert!(err.starts_with("NotClaimable:")); // Cannot claim before end

        let (bid2, _) = auction.submit_bid(&storage, 50_000, 500, 6000).unwrap();
        // Flow rate 2 = 50000 * 1e9 / 5000 = 10 * 1e9
        assert_eq!(bid2.flow_rate, 10 * RATE_PRECISION * price_precision);
        assert_eq!(auction.current_flow_rate, bid1.flow_rate + bid2.flow_rate);
        // bid1.flow_rate * 5000 * One token / (RATE_PRECISION * Price 100) = 250 tokens
        assert_eq!(auction.cumulative_supply_released, 250 * 100_000_000);
        // new supply rate = (1000 - 250) tokens / 5000 ms = 0.15 tokens/ms
        assert_eq!(auction.supply_rate, 15_000_000 * RATE_PRECISION); // 0.15 tokens / ms
        // Price = ((5 + 10) * 1e9 * 1e8) / (15_000_000 * 1e9) = 100
        assert_eq!(auction.get_clearing_price(), 100 * price_precision);
        // Acc 1 = (6000 - 1000) * 1e18 / 100 = 50 * 1e18 // price before bid2
        assert_eq!(
            auction.acc_tokens_per_share, // 50000000000
            50 * ACC_PRECISION / price_precision
        );
        assert_eq!(auction.acc_tokens_per_share, bid2.acc_snapshot);
        assert!(!auction.is_graduated());

        let (bid3, _) = auction.submit_bid(&storage, 50_000, 500, 9000).unwrap();
        // Flow rate 3 = 50000 * 1e9 / 2000 = 25 * 1e9
        assert_eq!(bid3.flow_rate, 25 * RATE_PRECISION * price_precision);

        assert_eq!(
            auction.current_flow_rate,
            bid1.flow_rate + bid2.flow_rate + bid3.flow_rate
        );
        // 250 + (bid1.flow_rate + bid2.flow_rate) * 3000 * One token / (RATE_PRECISION * Price 100) = 250 tokens
        assert_eq!(
            auction.cumulative_supply_released,
            (250 + 450) * 100_000_000
        );
        // new supply rate = (1000 - 250 - 450) tokens / 2000 ms = 0.15 tokens/ms
        assert_eq!(auction.supply_rate, 15_000_000 * RATE_PRECISION); // 0.15 tokens / ms
        // Price = ((5 + 10 + 25) * 1e9 * 1e8) / (15_000_000 * 1e9) = 400
        assert_eq!(auction.get_clearing_price(), 266666666667); // round up price_precision * 266.666...
        // Acc 2 = Acc 1 +  (9000 - 6000) * 1e18 / 100 = (50 + 30) * 1e18 // price before bid3
        assert_eq!(
            auction.acc_tokens_per_share, // 80000000000
            80 * ACC_PRECISION / price_precision
        );
        assert_eq!(auction.acc_tokens_per_share, bid3.acc_snapshot);
        assert!(!auction.is_graduated());

        let err = auction
            .claim(&storage, bid1.id, cfg.end_time - 1)
            .err()
            .unwrap();
        assert!(err.starts_with("NotClaimable:")); // Cannot claim before end
        assert!(auction.is_graduated());
        // Acc 3 = Acc 2 + (10999 - 9000) * 1e18 / 266666666667
        assert_eq!(
            auction.acc_tokens_per_share, // 8_749_6249_999
            80 * ACC_PRECISION / price_precision + 7_496_249_999
        );

        let err = auction
            .claim(&storage, bid1.id, cfg.end_time)
            .err()
            .unwrap();
        assert!(err.starts_with("NotClaimable:")); // Cannot claim before end
        assert!(auction.is_graduated());
        // Acc 4 = Acc 3 +  1 * 1e18 / 266666666667
        assert_eq!(
            auction.acc_tokens_per_share, // 87_499_999_999
            80 * ACC_PRECISION / price_precision + 7_496_249_999 + 3_750_000  // 3_749_999.999...
        );

        let bid1 = auction.claim(&storage, bid1.id, cfg.end_time + 1).unwrap();
        // println!("bid1: {:?}", bid1);
        // Spent = 5 * 1e9 * 10000 / 1e9 = 50000
        assert_eq!(bid1.refund, 0);
        // Tokens = (Acc 4 - 0) *  Flow rate 1 * one token / (1e9 * 1e18) =
        // 87_499_999_999 * (5 * 1e9 * 1e9) * 1e8 / (1e9 * 1e18)
        assert_eq!(bid1.tokens_filled, 43_749_999_999); // 43_749_999_999.5

        let bid2 = auction.claim(&storage, bid2.id, cfg.end_time + 1).unwrap();
        // println!("bid2: {:?}", bid2);
        // Spent = 10 * 1e9 * 5000 / 1e9 = 50000
        assert_eq!(bid2.refund, 0);
        // Tokens = (Acc 4 - Acc 1) *  Flow rate 2 * one token / (1e9 * 1e18) =
        // (87_499_999_999 - 50_000_000_000) * 10 * 1e9 * 1e9 * 1e8 / (1e9 * 1e18)
        assert_eq!(bid2.tokens_filled, 37_499_999_999);
        let bid3 = auction.claim(&storage, bid3.id, cfg.end_time + 1).unwrap();
        // println!("bid3: {:?}", bid3);
        // Spent = 25 * 1e9 * 2000 / 1e9 = 50000
        assert_eq!(bid3.refund, 0);
        // Tokens = (Acc 4 - Acc 2) *  Flow rate 3 * one token / (1e9 * 1e18) =
        // (87_499_999_999 - 80_000_000_000) * 25 * 1e9 * 1e9 * 1e8 / (1e9 * 1e18)
        assert_eq!(bid3.tokens_filled, 18_749_999_997); // 18_749_999_997.5

        let info = auction.get_info(cfg.end_time + 1);
        // println!("Auction Info: {:?}", info);
        assert_eq!(
            info.total_tokens_filled,
            bid1.tokens_filled + bid2.tokens_filled + bid3.tokens_filled
        );
        assert_eq!(
            info.cumulative_demand_raised,
            bid1.amount + bid2.amount + bid3.amount
        );
        assert!(info.cumulative_supply_released + 1 >= cfg.total_supply);
        assert!(info.cumulative_supply_released <= cfg.total_supply);
        assert_eq!(info.clearing_price, 266);
        // AuctionInfo { auction: AuctionConfig { start_time: 1000, end_time: 11000, min_bid_duration: 100, token_decimals: 8, total_supply: 100000000000, min_amount: 1000, max_amount: 1000000000, required_currency_raised: 100000 }, timestamp: 11001, clearing_price: 0, total_amount: 150000, total_tokens_filled: 99999999995, total_refunded: 0, cumulative_demand_raised: 150000, cumulative_supply_released: 99999999999, is_graduated: true }
    }

    #[test]
    fn test_outbid_logic() {
        let cfg = get_test_config();
        let mut auction = Auction::new(cfg.clone(), 8).unwrap();
        let storage = MockBidStorage::default();

        // User 1: Low price, early
        let (_, max_price) = auction.estimate_max_price(20_000, 1000); // 120
        let (bid1, _) = auction
            .submit_bid(&storage, 20_000, max_price, 1000)
            .unwrap();
        assert!(bid1.outbid_time.is_none());

        // Advance halfway
        let mid_time = 6000; // 5000ms passed
        let (bid2, _) = auction
            .submit_bid(&storage, 200_000, 1000, mid_time)
            .unwrap();

        // Check if bid1 is outbid
        let _bid1 = storage.get(bid1.id).unwrap();
        assert_eq!(_bid1.outbid_time, Some(mid_time));

        // Check bid2 is active
        let _bid2 = storage.get(bid2.id).unwrap();
        assert!(_bid2.outbid_time.is_none());

        // End auction and claim
        let end_time = cfg.end_time + 1;
        let bid1 = auction.claim(&storage, bid1.id, end_time).unwrap();
        // println!("Bid1 Claimed: {:?}", bid1);
        assert_eq!(bid1.outbid_time, Some(mid_time));
        assert_eq!(bid1.refund, 10_000);
        assert_eq!(bid1.tokens_filled, 10_000_000_000);
        let bid2 = auction.claim(&storage, bid2.id, end_time).unwrap();
        // println!("Bid2 Claimed: {:?}", bid2);
        assert_eq!(bid2.outbid_time, None);
        assert_eq!(bid2.refund, 0);
        assert_eq!(bid2.tokens_filled, 89_999_999_996);

        let info = auction.get_info(end_time);
        // println!("Auction Info: {:?}", info);
        assert_eq!(
            info.total_tokens_filled,
            bid1.tokens_filled + bid2.tokens_filled
        );
        assert_eq!(
            info.cumulative_demand_raised,
            bid1.amount + bid2.amount - bid1.refund
        );
        assert!(info.cumulative_supply_released + 1 >= cfg.total_supply);
        assert!(info.cumulative_supply_released <= cfg.total_supply);
        assert_eq!(info.clearing_price, 222);
        // AuctionInfo { auction: AuctionConfig { start_time: 1000, end_time: 11000, min_bid_duration: 100, token_decimals: 8, total_supply: 100000000000, min_amount: 1000, max_amount: 1000000000, required_currency_raised: 100000 }, timestamp: 11001, clearing_price: 222, total_amount: 220000, total_tokens_filled: 99999999996, total_refunded: 10000, cumulative_demand_raised: 210000, cumulative_supply_released: 99999999999, is_graduated: true }
    }

    #[test]
    fn test_auction_failure() {
        let cfg = get_test_config();
        let mut auction = Auction::new(cfg.clone(), 8).unwrap();
        let storage = MockBidStorage::default();

        // User 1: Low price, early
        let (bid1, _) = auction.submit_bid(&storage, 10_000, 200, 1000).unwrap();

        let (bid2, _) = auction.submit_bid(&storage, 50_000, 500, 2000).unwrap();

        let (bid3, _) = auction.submit_bid(&storage, 20_000, 500, 3000).unwrap();
        let err = auction
            .claim(&storage, bid1.id, cfg.end_time)
            .err()
            .unwrap();
        assert!(err.starts_with("NotClaimable:")); // Cannot claim before end

        // End auction and claim
        let end_time = cfg.end_time + 1;
        let bid1 = auction.claim(&storage, bid1.id, end_time).unwrap();
        assert_eq!(bid1.refund, 10_000);
        assert_eq!(bid1.tokens_filled, 0);
        assert_eq!(bid1.claim_time, end_time);

        let bid2 = auction.claim(&storage, bid2.id, end_time).unwrap();
        assert_eq!(bid2.refund, 50_000);
        assert_eq!(bid2.tokens_filled, 0);
        assert_eq!(bid2.claim_time, end_time);

        let bid3 = auction.claim(&storage, bid3.id, end_time).unwrap();
        assert_eq!(bid3.refund, 20_000);
        assert_eq!(bid3.tokens_filled, 0);
        assert_eq!(bid3.claim_time, end_time);

        let info = auction.get_info(end_time);
        // println!("Auction Info: {:?}", info);
        // AuctionInfo { auction: AuctionConfig { start_time: 1000, end_time: 11000, min_bid_duration: 100, token_decimals: 8, total_supply: 100000000000, min_amount: 1000, max_amount: 1000000000, required_currency_raised: 100000 }, timestamp: 11001, clearing_price: 100, total_amount: 80000, total_tokens_filled: 0, total_refunded: 80000, cumulative_demand_raised: 79999, cumulative_supply_released: 79999999999, is_graduated: false }
        assert_eq!(info.total_tokens_filled, 0);
        assert_eq!(info.total_refunded, 80_000);
        assert_eq!(info.clearing_price, 100);
        assert_eq!(info.cumulative_demand_raised, 79999);
        assert_eq!(info.cumulative_supply_released, 79999999999);
        assert!(!info.is_graduated);
    }
}
