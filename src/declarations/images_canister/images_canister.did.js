export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
  });
  const CanisterArgs = IDL.Variant({ 'Upgrade' : InitArgs, 'Init' : InitArgs });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const ImageMetadata = IDL.Record({
    'updated_at' : IDL.Nat64,
    'updated_by' : IDL.Principal,
    'name' : IDL.Text,
    'size' : IDL.Nat64,
    'type' : IDL.Text,
    'created_at' : IDL.Nat64,
    'locations' : IDL.Vec(IDL.Text),
  });
  const Result_1 = IDL.Variant({ 'Ok' : ImageMetadata, 'Err' : IDL.Text });
  const StateInfo = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
    'total_images' : IDL.Nat64,
    'tokens_canister' : IDL.Principal,
  });
  const Result_2 = IDL.Variant({ 'Ok' : StateInfo, 'Err' : IDL.Text });
  const ImageInput = IDL.Record({
    'body' : IDL.Vec(IDL.Nat8),
    'name' : IDL.Text,
    'type' : IDL.Text,
  });
  const Result_3 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  return IDL.Service({
    'admin_set_tokens_canister' : IDL.Func([IDL.Principal], [Result], []),
    'get_image_metadata' : IDL.Func([IDL.Nat64], [Result_1], ['query']),
    'info' : IDL.Func([], [Result_2], ['query']),
    'update_image' : IDL.Func([IDL.Nat64, ImageInput], [Result_3], []),
    'validate_admin_set_tokens_canister' : IDL.Func(
        [IDL.Principal],
        [Result_4],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'governance_canister' : IDL.Opt(IDL.Principal),
  });
  const CanisterArgs = IDL.Variant({ 'Upgrade' : InitArgs, 'Init' : InitArgs });
  return [IDL.Opt(CanisterArgs)];
};
