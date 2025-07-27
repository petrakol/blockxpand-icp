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
const summaryBar = document.getElementById("summary-bar");
const summaryDrawer = document.getElementById("summary-drawer");
const summaryTotal = document.getElementById("summary-total");
const spinner = document.getElementById("logo-spinner");
const skeleton = document.getElementById("summary-skeleton");
const errorDiv = document.getElementById("error-message");

function showError(msg) {
  errorDiv.textContent = msg;
  errorDiv.classList.remove("hidden");
  setTimeout(() => errorDiv.classList.add("hidden"), 5000);
}

function showLoading() {
  spinner.classList.remove("hidden");
  skeleton.classList.remove("hidden");
}

function hideLoading() {
  spinner.classList.add("hidden");
  skeleton.classList.add("hidden");
}

async function init() {
  authClient = await AuthClient.create();
  document.getElementById("connectBtn").addEventListener("click", connect);
  summaryBar.addEventListener("click", () => {
    const expanded = summaryDrawer.classList.toggle("open");
    summaryDrawer.classList.remove("hidden");
    summaryBar.setAttribute("aria-expanded", expanded);
    summaryBar.classList.toggle("expanded", expanded);
  });
  if (await authClient.isAuthenticated()) {
    showLoading();
    await onConnect();
    hideLoading();
  }
}

async function connect() {
  showLoading();
  if (window.ic && window.ic.plug) {
    try {
      await window.ic.plug.requestConnect({ whitelist: [window.CANISTER_ID] });
      actor = await window.ic.plug.createActor({
        canisterId: window.CANISTER_ID,
        interfaceFactory: idlFactory,
      });
      document.getElementById("principal").textContent = window.ic.plug.principalId;
      document.getElementById("connectBtn").classList.add("hidden");
      summaryBar.classList.remove("hidden");
      fetchSummary();
      hideLoading();
      return;
    } catch (e) {
      console.error("Plug connect failed", e);
      showError("Connection failed. Please check your Internet Identity and try again.");
      hideLoading();
    }
  }
  try {
    await authClient.login({
      identityProvider: "https://identity.ic0.app/#authorize",
      onSuccess: async () => {
        await onConnect();
        hideLoading();
      },
    });
  } catch (e) {
    console.error("II connect failed", e);
    showError("Connection failed. Please check your Internet Identity and try again.");
    hideLoading();
  }
}

async function onConnect() {
  const identity = await authClient.getIdentity();
  const agent = new HttpAgent({ identity });
  actor = Actor.createActor(idlFactory, { agent, canisterId: window.CANISTER_ID });
  document.getElementById("principal").textContent = identity.getPrincipal().toText();
  document.getElementById("connectBtn").classList.add("hidden");
  summaryBar.classList.remove("hidden");
  fetchSummary();
}

async function fetchSummary() {
  const principal = actor.agent && actor.agent.identity ? actor.agent.identity.getPrincipal() : null;
  showLoading();
  try {
    const res = await actor.get_holdings_summary(
      principal || window.ic.plug.principalId
    );
    if ("Ok" in res) {
      const details = res.Ok.map((item) => [item.token, item.total]);
      const total = details.reduce((acc, [, amt]) => acc + amt, 0);
      populateSummary(total, details);
    } else if ("Err" in res) {
      if (/cycles/i.test(res.Err)) {
        showError("Insufficient cycles attached. Please top up your wallet and retry.");
      } else {
        showError("Unable to load your balances. Please try again later.");
      }
    }
  } catch (e) {
    if (/cycles/i.test(e.message || "")) {
      showError("Insufficient cycles attached. Please top up your wallet and retry.");
    } else {
      showError("Unable to load your balances. Please try again later.");
    }
  } finally {
    hideLoading();
  }
}

function populateSummary(totalAmount, details) {
  summaryTotal.textContent = `Total Rewards: ${totalAmount.toFixed(4)} ICP`;
  let html = "<h2>Your balances</h2><ul>";
  details.forEach(([token, amount]) => {
    html += `<li>${token}: ${amount.toFixed(4)}</li>`;
  });
  html += "</ul>";
  summaryDrawer.innerHTML = html;
  summaryBar.classList.remove("hidden");
}

init();
