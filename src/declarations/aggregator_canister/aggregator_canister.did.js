export const idlFactory = ({ IDL }) => {
  const TokenTotal = IDL.Record({ token: IDL.Text, total: IDL.Float64 });
  return IDL.Service({
    get_holdings_summary: IDL.Func([IDL.Principal], [IDL.Variant({ Ok: IDL.Vec(TokenTotal), Err: IDL.Text })], []),
  });
};

export const canisterId = "<CANISTER_ID>";
