export const idlFactory = ({ IDL }) => {
  const UpgradeArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
  });
  const InitArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'key_name' : IDL.Text,
  });
  const CanisterArgs = IDL.Variant({
    'Upgrade' : UpgradeArgs,
    'Init' : InitArgs,
  });
  const AuctionId = IDL.Variant({
    'Evm' : IDL.Text,
    'Icp' : IDL.Text,
    'Sol' : IDL.Text,
  });
  const Chain = IDL.Variant({
    'Evm' : IDL.Nat64,
    'Icp' : IDL.Nat64,
    'Sol' : IDL.Nat64,
  });
  const AuctionInfo = IDL.Record({
    'id' : AuctionId,
    'url' : IDL.Text,
    'token' : IDL.Text,
    'token_logo_url' : IDL.Text,
    'total_demand_raised' : IDL.Nat,
    'token_symbol' : IDL.Text,
    'chain' : Chain,
    'name' : IDL.Text,
    'currency_name' : IDL.Text,
    'update_time' : IDL.Nat64,
    'currency_symbol' : IDL.Text,
    'description' : IDL.Text,
    'end_time' : IDL.Nat64,
    'bids_count' : IDL.Nat64,
    'total_supply_released' : IDL.Nat,
    'start_time' : IDL.Nat64,
    'currency_decimals' : IDL.Nat8,
    'currency' : IDL.Text,
    'clearing_price' : IDL.Nat,
    'token_decimals' : IDL.Nat8,
    'token_name' : IDL.Text,
    'is_graduated' : IDL.Bool,
    'currency_logo_url' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const StateInfo = IDL.Record({
    'sol_address' : IDL.Text,
    'evm_address' : IDL.Text,
    'governance_canister' : IDL.Opt(IDL.Principal),
    'daos' : IDL.Vec(IDL.Principal),
    'auctions' : IDL.Vec(AuctionId),
    'icp_address' : IDL.Principal,
    'chain_providers' : IDL.Vec(IDL.Tuple(Chain, IDL.Vec(IDL.Text))),
    'key_name' : IDL.Text,
    'paying_public_keys' : IDL.Vec(IDL.Vec(IDL.Nat8)),
    'storages' : IDL.Vec(IDL.Principal),
  });
  const Result_1 = IDL.Variant({ 'Ok' : StateInfo, 'Err' : IDL.Text });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  return IDL.Service({
    'admin_set_auction' : IDL.Func([AuctionInfo], [Result], []),
    'admin_set_paying_public_keys' : IDL.Func(
        [IDL.Vec(IDL.Text)],
        [Result],
        [],
      ),
    'admin_set_providers' : IDL.Func([Chain, IDL.Vec(IDL.Text)], [Result], []),
    'get_auction' : IDL.Func(
        [IDL.Opt(AuctionId)],
        [IDL.Opt(AuctionInfo)],
        ['query'],
      ),
    'info' : IDL.Func([], [Result_1], ['query']),
    'list_auctions' : IDL.Func(
        [IDL.Nat64, IDL.Opt(AuctionId)],
        [IDL.Vec(AuctionInfo)],
        ['query'],
      ),
    'validate_admin_set_auction' : IDL.Func([AuctionInfo], [Result_2], []),
    'validate_admin_set_paying_public_keys' : IDL.Func(
        [IDL.Vec(IDL.Text)],
        [Result_2],
        [],
      ),
    'validate_admin_set_providers' : IDL.Func(
        [Chain, IDL.Vec(IDL.Text)],
        [Result_2],
        [],
      ),
    'validate_empty_input' : IDL.Func([], [Result_2], []),
  });
};
export const init = ({ IDL }) => {
  const UpgradeArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
  });
  const InitArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'key_name' : IDL.Text,
  });
  const CanisterArgs = IDL.Variant({
    'Upgrade' : UpgradeArgs,
    'Init' : InitArgs,
  });
  return [IDL.Opt(CanisterArgs)];
};
