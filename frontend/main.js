import { HttpAgent, Actor } from "https://unpkg.com/@dfinity/agent@0.18.0/dist/agent.min.js";
import { AuthClient } from "https://unpkg.com/@dfinity/auth-client@0.18.0/dist/auth-client.min.js";
import { idlFactory, canisterId } from "../src/declarations/aggregator_canister/aggregator_canister.did.js";

const connectBtn = document.getElementById('connectBtn');
const summaryBar = document.getElementById('summary-bar');
const summaryDrawer = document.getElementById('summary-drawer');
const summaryTotal = document.getElementById('summary-total');
const spinner = document.getElementById('logo-spinner');
const skeleton = document.getElementById('summary-skeleton');
const errorDiv = document.getElementById('error-message');

let authClient = null;
let actor = null;

function showError(msg) {
  errorDiv.textContent = msg;
  errorDiv.classList.remove('hidden');
  setTimeout(() => errorDiv.classList.add('hidden'), 5000);
}

function showLoading() {
  spinner.classList.remove('hidden');
  skeleton.classList.remove('hidden');
}

function hideLoading() {
  spinner.classList.add('hidden');
  skeleton.classList.add('hidden');
}

function isMainnet() {
  return (window.DFX_NETWORK || 'local') === 'ic';
}

async function createActor() {
  const identity = await authClient.getIdentity();
  const agent = new HttpAgent({ identity });
  if (!isMainnet()) {
    await agent.fetchRootKey();
  }
  actor = Actor.createActor(idlFactory, { agent, canisterId: window.CANISTER_ID || canisterId });
}

async function loadSummary() {
  summaryBar.textContent = 'Loading...';
  try {
    const principal = (await authClient.getIdentity()).getPrincipal().toText();
    const res = await actor.get_holdings_summary(principal);
    if ('Ok' in res) {
      const total = res.Ok.reduce((acc, t) => acc + t.total, 0);
      summaryTotal.textContent = `Total Rewards: ${total.toFixed(4)} ICP`;
      summaryDrawer.innerHTML = res.Ok.map(t => `<div>${t.token}: ${t.total.toFixed(4)}</div>`).join('');
    } else {
      showError(res.Err);
    }
  } catch (err) {
    showError('Failed to load your summary. ' + err.message);
  } finally {
    hideLoading();
  }
}

async function init() {
  authClient = await AuthClient.create();

  connectBtn.addEventListener('click', async () => {
    await authClient.login({
      identityProvider: 'https://identity.ic0.app/#authorize',
      onSuccess: async () => {
        await createActor();
        await loadSummary();
      },
    });
  });

  summaryBar.addEventListener('click', () => {
    const expanded = summaryDrawer.classList.toggle('open');
    summaryBar.setAttribute('aria-expanded', expanded);
  });

  if (await authClient.isAuthenticated()) {
    await createActor();
    await loadSummary();
  }
}

showLoading();
init();
