export const idlFactory = ({ IDL }) => {
  const UpgradeArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
  });
  const Chain = IDL.Variant({
    'Evm' : IDL.Nat64,
    'Icp' : IDL.Null,
    'Sol' : IDL.Null,
  });
  const InitArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'chain' : Chain,
    'key_name' : IDL.Text,
  });
  const CanisterArgs = IDL.Variant({
    'Upgrade' : UpgradeArgs,
    'Init' : InitArgs,
  });
  const FinalizeOutput = IDL.Record({
    'txid' : IDL.Text,
    'pool_id' : IDL.Text,
  });
  const Result = IDL.Variant({
    'Ok' : IDL.Opt(FinalizeOutput),
    'Err' : IDL.Text,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const AuctionConfig = IDL.Record({
    'min_amount' : IDL.Nat,
    'liquidity_pool_amount' : IDL.Nat,
    'end_time' : IDL.Nat64,
    'start_time' : IDL.Nat64,
    'token_decimals' : IDL.Nat8,
    'required_currency_raised' : IDL.Nat,
    'max_amount' : IDL.Nat,
    'total_supply' : IDL.Nat,
    'min_bid_duration' : IDL.Nat64,
  });
  const TokenInput = IDL.Record({
    'decimals' : IDL.Nat8,
    'token' : IDL.Text,
    'name' : IDL.Text,
    'program_id' : IDL.Opt(IDL.Text),
    'recipient' : IDL.Text,
    'logo_url' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const FinalizeKind = IDL.Variant({
    'CreatePool' : IDL.Text,
    'Transfer' : IDL.Null,
  });
  const ProjectInput = IDL.Record({
    'url' : IDL.Text,
    'name' : IDL.Text,
    'description' : IDL.Text,
    'persons_excluded' : IDL.Vec(IDL.Text),
  });
  const WithdrawTxInfo = IDL.Record({
    'id' : IDL.Nat64,
    'kind' : IDL.Nat8,
    'txid' : IDL.Text,
    'user' : IDL.Principal,
    'recipient' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'amount' : IDL.Nat,
  });
  const Result_2 = IDL.Variant({ 'Ok' : WithdrawTxInfo, 'Err' : IDL.Text });
  const AuctionInfo = IDL.Record({
    'cumulative_demand_raised' : IDL.Nat,
    'total_amount' : IDL.Nat,
    'total_tokens_filled' : IDL.Nat,
    'timestamp' : IDL.Nat64,
    'clearing_price' : IDL.Nat,
    'bidders_count' : IDL.Nat64,
    'total_refunded' : IDL.Nat,
    'is_graduated' : IDL.Bool,
    'cumulative_supply_released' : IDL.Nat,
  });
  const BidInfo = IDL.Record({
    'id' : IDL.Nat64,
    'tokens_filled' : IDL.Nat,
    'outbid_time' : IDL.Opt(IDL.Nat64),
    'acc_snapshot' : IDL.Nat,
    'create_time' : IDL.Nat64,
    'claim_time' : IDL.Nat64,
    'outbid_acc_snapshot' : IDL.Opt(IDL.Nat),
    'max_price' : IDL.Nat,
    'flow_rate' : IDL.Nat,
    'amount' : IDL.Nat,
    'refund' : IDL.Nat,
  });
  const Result_3 = IDL.Variant({ 'Ok' : BidInfo, 'Err' : IDL.Text });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Vec(BidInfo), 'Err' : IDL.Text });
  const DepositInput = IDL.Record({ 'txid' : IDL.Text, 'sender' : IDL.Text });
  const Result_5 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : IDL.Text });
  const AuctionSnapshot = IDL.Record({
    'c' : IDL.Nat,
    'd' : IDL.Nat,
    'f' : IDL.Nat,
    's' : IDL.Nat,
    't' : IDL.Nat64,
  });
  const StateInfo = IDL.Record({
    'url' : IDL.Text,
    'token' : IDL.Text,
    'sol_address' : IDL.Text,
    'token_program_id' : IDL.Opt(IDL.Text),
    'evm_address' : IDL.Text,
    'total_withdrawn_token' : IDL.Nat,
    'token_logo_url' : IDL.Text,
    'token_symbol' : IDL.Text,
    'governance_canister' : IDL.Opt(IDL.Principal),
    'chain' : Chain,
    'name' : IDL.Text,
    'currency_name' : IDL.Text,
    'total_bidders' : IDL.Nat64,
    'icp_address' : IDL.Principal,
    'currency_symbol' : IDL.Text,
    'description' : IDL.Text,
    'chain_providers' : IDL.Vec(IDL.Text),
    'currency_decimals' : IDL.Nat8,
    'funds_recipient' : IDL.Text,
    'tokens_recipient' : IDL.Text,
    'auction_config' : IDL.Opt(AuctionConfig),
    'currency' : IDL.Text,
    'persons_excluded' : IDL.Vec(IDL.Text),
    'finalize_kind' : FinalizeKind,
    'key_name' : IDL.Text,
    'token_decimals' : IDL.Nat8,
    'total_deposited_currency' : IDL.Nat,
    'token_name' : IDL.Text,
    'auction_finalized' : IDL.Bool,
    'total_withdrawn_currency' : IDL.Nat,
    'currency_program_id' : IDL.Opt(IDL.Text),
    'currency_logo_url' : IDL.Text,
  });
  const Result_6 = IDL.Variant({ 'Ok' : StateInfo, 'Err' : IDL.Text });
  const DepositTxInfo = IDL.Record({
    'txid' : IDL.Text,
    'user' : IDL.Principal,
    'sender' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'amount' : IDL.Nat,
  });
  const Result_7 = IDL.Variant({
    'Ok' : IDL.Vec(DepositTxInfo),
    'Err' : IDL.Text,
  });
  const Result_8 = IDL.Variant({
    'Ok' : IDL.Vec(WithdrawTxInfo),
    'Err' : IDL.Text,
  });
  const Result_9 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const WithdrawInput = IDL.Record({ 'recipient' : IDL.Text });
  const PayingResultInput = IDL.Record({
    'result' : IDL.Vec(IDL.Nat8),
    'signature' : IDL.Vec(IDL.Nat8),
    'timestamp' : IDL.Nat64,
  });
  const X402PaymentOutput = IDL.Record({
    'x402' : IDL.Vec(IDL.Nat8),
    'nonce' : IDL.Text,
    'timestamp' : IDL.Nat64,
  });
  const Result_10 = IDL.Variant({ 'Ok' : X402PaymentOutput, 'Err' : IDL.Text });
  return IDL.Service({
    'admin_finalize_auction' : IDL.Func([], [Result], []),
    'admin_init_auction' : IDL.Func([], [Result_1], []),
    'admin_set_auction' : IDL.Func([AuctionConfig], [Result_1], []),
    'admin_set_currency' : IDL.Func([TokenInput], [Result_1], []),
    'admin_set_finalize' : IDL.Func([FinalizeKind], [Result_1], []),
    'admin_set_paying_public_keys' : IDL.Func(
        [IDL.Vec(IDL.Text)],
        [Result_1],
        [],
      ),
    'admin_set_project' : IDL.Func([ProjectInput], [Result_1], []),
    'admin_set_providers' : IDL.Func([IDL.Vec(IDL.Text)], [Result_1], []),
    'admin_set_token' : IDL.Func([TokenInput], [Result_1], []),
    'admin_sweep_currency' : IDL.Func([], [Result_2], []),
    'admin_sweep_token' : IDL.Func([], [Result_2], []),
    'auction_info' : IDL.Func([], [IDL.Opt(AuctionInfo)], ['query']),
    'claim' : IDL.Func([IDL.Nat64], [Result_3], []),
    'claim_all' : IDL.Func([], [Result_4], []),
    'deposit_currency' : IDL.Func([DepositInput], [Result_5], []),
    'estimate_max_price' : IDL.Func([IDL.Nat], [IDL.Nat], ['query']),
    'get_grouped_bids' : IDL.Func(
        [IDL.Opt(IDL.Nat64)],
        [IDL.Vec(IDL.Tuple(IDL.Nat, IDL.Nat))],
        ['query'],
      ),
    'get_snapshots' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(AuctionSnapshot)],
        ['query'],
      ),
    'info' : IDL.Func([], [Result_6], ['query']),
    'my_bids' : IDL.Func([], [Result_4], ['query']),
    'my_deposits' : IDL.Func([], [Result_7], ['query']),
    'my_withdraws' : IDL.Func([], [Result_8], ['query']),
    'submit_bid' : IDL.Func([IDL.Nat, IDL.Nat], [Result_3], []),
    'validate_admin_set_currency' : IDL.Func([TokenInput], [Result_9], []),
    'validate_admin_set_finalize' : IDL.Func([FinalizeKind], [Result_9], []),
    'validate_admin_set_paying_public_keys' : IDL.Func(
        [IDL.Vec(IDL.Text)],
        [Result_9],
        [],
      ),
    'validate_admin_set_project' : IDL.Func([ProjectInput], [Result_9], []),
    'validate_admin_set_providers' : IDL.Func(
        [IDL.Vec(IDL.Text)],
        [Result_9],
        [],
      ),
    'validate_admin_set_token' : IDL.Func([TokenInput], [Result_9], []),
    'validate_empty_input' : IDL.Func([], [Result_9], []),
    'withdraw_currency' : IDL.Func([WithdrawInput], [Result_2], []),
    'withdraw_token' : IDL.Func([WithdrawInput], [Result_2], []),
    'x402_bind_address' : IDL.Func([PayingResultInput], [Result_1], []),
    'x402_deposit_currency' : IDL.Func([PayingResultInput], [Result_5], []),
    'x402_payment' : IDL.Func([IDL.Nat, IDL.Bool], [Result_10], ['query']),
  });
};
export const init = ({ IDL }) => {
  const UpgradeArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
  });
  const Chain = IDL.Variant({
    'Evm' : IDL.Nat64,
    'Icp' : IDL.Null,
    'Sol' : IDL.Null,
  });
  const InitArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'chain' : Chain,
    'key_name' : IDL.Text,
  });
  const CanisterArgs = IDL.Variant({
    'Upgrade' : UpgradeArgs,
    'Init' : InitArgs,
  });
  return [IDL.Opt(CanisterArgs)];
};
