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
  const AuctionConfig = IDL.Record({
    'min_amount' : IDL.Nat,
    'end_time' : IDL.Nat64,
    'start_time' : IDL.Nat64,
    'token_decimals' : IDL.Nat8,
    'required_currency_raised' : IDL.Nat,
    'max_amount' : IDL.Nat,
    'total_supply' : IDL.Nat,
    'min_bid_duration' : IDL.Nat64,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const TokenInput = IDL.Record({
    'decimals' : IDL.Nat8,
    'token' : IDL.Text,
    'program_id' : IDL.Opt(IDL.Text),
    'recipient' : IDL.Text,
  });
  const AuctionInfo = IDL.Record({
    'cumulative_demand_raised' : IDL.Nat,
    'total_amount' : IDL.Nat,
    'total_tokens_filled' : IDL.Nat,
    'timestamp' : IDL.Nat64,
    'clearing_price' : IDL.Nat,
    'bidders_count' : IDL.Nat64,
    'total_refunded' : IDL.Nat,
    'auction' : AuctionConfig,
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
  const Result_1 = IDL.Variant({ 'Ok' : BidInfo, 'Err' : IDL.Text });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Vec(BidInfo), 'Err' : IDL.Text });
  const DepositInput = IDL.Record({ 'txid' : IDL.Text, 'sender' : IDL.Text });
  const Result_3 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : IDL.Text });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const StateInfo = IDL.Record({
    'token' : IDL.Text,
    'sol_address' : IDL.Text,
    'token_program_id' : IDL.Opt(IDL.Text),
    'evm_address' : IDL.Text,
    'governance_canister' : IDL.Opt(IDL.Principal),
    'chain' : Chain,
    'icp_address' : IDL.Principal,
    'chain_providers' : IDL.Vec(IDL.Text),
    'currency_decimals' : IDL.Nat8,
    'funds_recipient' : IDL.Text,
    'tokens_recipient' : IDL.Text,
    'currency' : IDL.Text,
    'key_name' : IDL.Text,
    'token_decimals' : IDL.Nat8,
    'currency_program_id' : IDL.Opt(IDL.Text),
  });
  const Result_5 = IDL.Variant({ 'Ok' : StateInfo, 'Err' : IDL.Text });
  const WithdrawInput = IDL.Record({ 'recipient' : IDL.Text });
  const WithdrawTxInfo = IDL.Record({
    'id' : IDL.Nat64,
    'kind' : IDL.Nat8,
    'txid' : IDL.Text,
    'user' : IDL.Principal,
    'recipient' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'amount' : IDL.Nat,
  });
  const Result_6 = IDL.Variant({ 'Ok' : WithdrawTxInfo, 'Err' : IDL.Text });
  return IDL.Service({
    'admin_set_auction' : IDL.Func([AuctionConfig], [Result], []),
    'admin_set_currency' : IDL.Func([TokenInput], [Result], []),
    'admin_set_providers' : IDL.Func([IDL.Vec(IDL.Text)], [Result], []),
    'admin_set_token' : IDL.Func([TokenInput], [Result], []),
    'auction_info' : IDL.Func([], [IDL.Opt(AuctionInfo)], ['query']),
    'claim' : IDL.Func([IDL.Nat64], [Result_1], []),
    'claim_all' : IDL.Func([], [Result_2], []),
    'deposit_currency' : IDL.Func([DepositInput], [Result_3], []),
    'estimate_max_price' : IDL.Func([IDL.Nat], [IDL.Nat], ['query']),
    'evm_address' : IDL.Func([IDL.Opt(IDL.Principal)], [Result_4], ['query']),
    'get_grouped_bids' : IDL.Func(
        [IDL.Opt(IDL.Nat64)],
        [IDL.Vec(IDL.Tuple(IDL.Nat, IDL.Nat))],
        ['query'],
      ),
    'info' : IDL.Func([], [Result_5], ['query']),
    'my_bids' : IDL.Func([], [Result_2], ['query']),
    'sol_address' : IDL.Func([IDL.Opt(IDL.Principal)], [Result_4], ['query']),
    'submit_bid' : IDL.Func([IDL.Nat, IDL.Nat], [Result_1], []),
    'validate_admin_set_currency' : IDL.Func([TokenInput], [Result_4], []),
    'validate_admin_set_providers' : IDL.Func(
        [IDL.Vec(IDL.Text)],
        [Result_4],
        [],
      ),
    'validate_admin_set_token' : IDL.Func([TokenInput], [Result_4], []),
    'withdraw_currency' : IDL.Func([WithdrawInput], [Result_6], []),
    'withdraw_token' : IDL.Func([WithdrawInput], [Result_6], []),
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
