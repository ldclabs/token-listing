export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
  });
  const CanisterArgs = IDL.Variant({ 'Upgrade' : InitArgs, 'Init' : InitArgs });
  const LinkType = IDL.Variant({
    'Homepage' : IDL.Null,
    'Bridge' : IDL.Null,
    'Social' : IDL.Null,
    'SourceCode' : IDL.Null,
    'Whitepaper' : IDL.Null,
    'Browser' : IDL.Null,
    'Documentation' : IDL.Null,
    'Exchange' : IDL.Null,
    'Audit' : IDL.Null,
    'Governance' : IDL.Null,
  });
  const LinkItem = IDL.Record({
    'rel' : LinkType,
    'url' : IDL.Text,
    'name' : IDL.Text,
  });
  const TokenMetadata = IDL.Record({
    'decimals' : IDL.Nat8,
    'external_url' : IDL.Text,
    'name' : IDL.Text,
    'description' : IDL.Text,
    'links' : IDL.Vec(LinkItem),
    'locations' : IDL.Vec(IDL.Text),
    'image' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Vec(IDL.Nat64), 'Err' : IDL.Text });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const TokenStatus = IDL.Variant({
    'Active' : IDL.Null,
    'Deprecated' : IDL.Null,
    'Locked' : IDL.Null,
  });
  const VerificationBadge = IDL.Record({
    'methods' : IDL.Vec(IDL.Text),
    'is_verified' : IDL.Bool,
    'verified_at' : IDL.Nat64,
  });
  const Announcement = IDL.Record({
    'id' : IDL.Nat64,
    'url' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'content' : IDL.Text,
    'type' : IDL.Text,
    'published_at' : IDL.Nat64,
  });
  const TokenProfile = IDL.Record({
    'id' : IDL.Nat64,
    'status' : TokenStatus,
    'updated_at' : IDL.Nat64,
    'controllers' : IDL.Vec(IDL.Principal),
    'metadata' : TokenMetadata,
    'tags' : IDL.Vec(IDL.Text),
    'created_at' : IDL.Nat64,
    'verification' : VerificationBadge,
    'announcements' : IDL.Vec(Announcement),
  });
  const Result_2 = IDL.Variant({ 'Ok' : TokenProfile, 'Err' : IDL.Text });
  const StateInfo = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'x402_prices' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat64)),
    'x402_pay_to' : IDL.Text,
    'total_tokens' : IDL.Nat64,
    'x402_paying_public_keys' : IDL.Vec(IDL.Vec(IDL.Nat8)),
  });
  const Result_3 = IDL.Variant({ 'Ok' : StateInfo, 'Err' : IDL.Text });
  const PayingResultInput = IDL.Record({
    'result' : IDL.Vec(IDL.Nat8),
    'signature' : IDL.Vec(IDL.Nat8),
    'timestamp' : IDL.Nat64,
  });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text });
  const Result_5 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const X402PaymentOutput = IDL.Record({
    'x402' : IDL.Vec(IDL.Nat8),
    'nonce' : IDL.Text,
    'timestamp' : IDL.Nat64,
  });
  const Result_6 = IDL.Variant({ 'Ok' : X402PaymentOutput, 'Err' : IDL.Text });
  return IDL.Service({
    'admin_batch_register_tokens' : IDL.Func(
        [IDL.Vec(TokenMetadata)],
        [Result],
        [],
      ),
    'admin_set_paying_public_keys' : IDL.Func(
        [IDL.Vec(IDL.Text)],
        [Result_1],
        [],
      ),
    'admin_set_payment_requirements_extra' : IDL.Func(
        [IDL.Text],
        [Result_1],
        [],
      ),
    'admin_set_x402_pay_to' : IDL.Func([IDL.Text], [Result_1], []),
    'admin_set_x402_prices' : IDL.Func(
        [IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat64))],
        [Result_1],
        [],
      ),
    'admin_update_token_status' : IDL.Func(
        [IDL.Nat64, TokenStatus],
        [Result_1],
        [],
      ),
    'admin_update_token_tags' : IDL.Func(
        [IDL.Nat64, IDL.Vec(IDL.Text)],
        [Result_1],
        [],
      ),
    'admin_update_token_verification_badge' : IDL.Func(
        [IDL.Nat64, VerificationBadge],
        [Result_1],
        [],
      ),
    'get_token_profile' : IDL.Func([IDL.Nat64], [Result_2], ['query']),
    'info' : IDL.Func([], [Result_3], ['query']),
    'list_tokens' : IDL.Func(
        [IDL.Nat64, IDL.Opt(IDL.Nat64)],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, TokenMetadata))],
        ['query'],
      ),
    'query_token' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, TokenMetadata))],
        ['query'],
      ),
    'register_token' : IDL.Func(
        [TokenMetadata, PayingResultInput],
        [Result_4],
        [],
      ),
    'set_announcement' : IDL.Func(
        [IDL.Nat64, Announcement, PayingResultInput],
        [Result_1],
        [],
      ),
    'set_location' : IDL.Func([IDL.Nat64, PayingResultInput], [Result_1], []),
    'update_token_controllers' : IDL.Func(
        [IDL.Nat64, IDL.Vec(IDL.Principal), PayingResultInput],
        [Result_1],
        [],
      ),
    'update_token_metadata' : IDL.Func(
        [IDL.Nat64, TokenMetadata, PayingResultInput],
        [Result_1],
        [],
      ),
    'validate_admin_set_paying_public_keys' : IDL.Func(
        [IDL.Vec(IDL.Text)],
        [Result_5],
        [],
      ),
    'validate_admin_set_payment_requirements_extra' : IDL.Func(
        [IDL.Text],
        [Result_5],
        [],
      ),
    'validate_admin_set_x402_pay_to' : IDL.Func([IDL.Text], [Result_5], []),
    'validate_admin_set_x402_prices' : IDL.Func(
        [IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat64))],
        [Result_5],
        [],
      ),
    'x402_payment' : IDL.Func([IDL.Text], [Result_6], ['query']),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
  });
  const CanisterArgs = IDL.Variant({ 'Upgrade' : InitArgs, 'Init' : InitArgs });
  return [IDL.Opt(CanisterArgs)];
};
