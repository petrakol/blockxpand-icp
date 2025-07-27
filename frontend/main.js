import { HttpAgent, Actor } from "https://unpkg.com/@dfinity/agent@1.1.1?module";
import { AuthClient } from "https://unpkg.com/@dfinity/auth-client@1.1.1?module";

const idlFactory = ({ IDL }) =>
  IDL.Service({
    get_holdings_summary: IDL.Func(
      [IDL.Principal],
      [IDL.Variant({
        Ok: IDL.Vec(IDL.Record({ token: IDL.Text, total: IDL.Float64 })),
        Err: IDL.Text,
      })],
      []
    ),
  });

let actor;
let authClient;

async function init() {
  authClient = await AuthClient.create();
  document.getElementById("connectBtn").addEventListener("click", connect);
  if (await authClient.isAuthenticated()) {
    await onConnect();
  }
}

async function connect() {
  if (window.ic && window.ic.plug) {
    try {
      await window.ic.plug.requestConnect({ whitelist: [window.CANISTER_ID] });
      actor = await window.ic.plug.createActor({
        canisterId: window.CANISTER_ID,
        interfaceFactory: idlFactory,
      });
      document.getElementById("principal").textContent = window.ic.plug.principalId;
      document.getElementById("connectBtn").classList.add("hidden");
      fetchSummary();
      return;
    } catch (e) {
      console.error("Plug connect failed", e);
    }
  }

  await authClient.login({
    identityProvider: "https://identity.ic0.app/#authorize",
    onSuccess: onConnect,
  });
}

async function onConnect() {
  const identity = await authClient.getIdentity();
  const agent = new HttpAgent({ identity });
  actor = Actor.createActor(idlFactory, { agent, canisterId: window.CANISTER_ID });
  document.getElementById("principal").textContent = identity.getPrincipal().toText();
  document.getElementById("connectBtn").classList.add("hidden");
  fetchSummary();
}

async function fetchSummary() {
  const principal = actor.agent && actor.agent.identity ? actor.agent.identity.getPrincipal() : null;
  const res = await actor.get_holdings_summary(principal || window.ic.plug.principalId);
  if ("Ok" in res) {
    const list = document.getElementById("summary");
    list.innerHTML = "";
    res.Ok.forEach((item) => {
      const div = document.createElement("div");
      div.className = "summary-item";
      div.textContent = `${item.token}: ${item.total.toFixed(4)}`;
      list.appendChild(div);
    });
    list.classList.remove("hidden");
  }
}

init();
