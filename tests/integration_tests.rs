#[cfg(test)]
mod tests {
    use blockxpand_icp::get_holdings;
    use blockxpand_icp::Holding;
    use candid::{Encode, Decode, Principal};
    use ic_agent::{identity::AnonymousIdentity, Agent};
    use std::process::Command;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn integration_get_holdings() {
        // Ensure `dfx` is installed for this test.
        if Command::new("dfx").arg("--version").output().is_err() {
            let _ = Command::new("./install_dfx.sh").status();
            if Command::new("dfx").arg("--version").output().is_err() {
                eprintln!("dfx not found; skipping integration test");
                return;
            }
        }

        // Start a local replica and ensure it is stopped at the end.
        Command::new("dfx")
            .args(["start", "--background", "--clean"])
            .status()
            .expect("failed to start dfx");
        struct Stop;
        impl Drop for Stop {
            fn drop(&mut self) {
                let _ = Command::new("dfx").arg("stop").status();
            }
        }
        let _stop = Stop;

        Command::new("dfx")
            .args(["deploy", "mock_ledger"])
            .status()
            .expect("failed to deploy mock ledger");

        let output = Command::new("dfx")
            .args(["canister", "id", "mock_ledger"])
            .output()
            .expect("failed to get ledger id");
        let cid = String::from_utf8(output.stdout).unwrap();
        let cid = cid.trim();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[ledgers]\nMOCK = \"{cid}\"").unwrap();

        std::env::set_var("LEDGER_URL", "http://127.0.0.1:4943");
        std::env::set_var("LEDGERS_FILE", file.path());

        let principal = Principal::anonymous();
        let holdings = get_holdings(principal).await;
        assert_eq!(holdings.len(), 4);
        assert_eq!(holdings[0].token, "MOCK");
        assert_eq!(holdings[0].status, "liquid");

        // Deploy the aggregator canister using the mock ledger.
        let cfg_path = std::path::Path::new("config/ledgers.toml");
        let original = std::fs::read_to_string(cfg_path).unwrap();
        std::fs::write(cfg_path, format!("[ledgers]\nICP = \"{cid}\"\n")).unwrap();
        struct Restore(String);
        impl Drop for Restore {
            fn drop(&mut self) {
                let _ = std::fs::write("config/ledgers.toml", &self.0);
            }
        }
        let _restore = Restore(original);

        Command::new("dfx")
            .args(["deploy", "aggregator"])
            .status()
            .expect("failed to deploy aggregator");

        let output = Command::new("dfx")
            .args(["canister", "id", "aggregator"])
            .output()
            .expect("failed to get aggregator id");
        let aggr_id = String::from_utf8(output.stdout).unwrap();
        let aggr_id = aggr_id.trim();

        let agent = Agent::builder()
            .with_url("http://127.0.0.1:4943")
            .with_identity(AnonymousIdentity {})
            .build()
            .unwrap();
        let _ = agent.fetch_root_key().await;
        let arg = candid::Encode!(&Principal::anonymous()).unwrap();
        let bytes = agent
            .query(&Principal::from_text(aggr_id).unwrap(), "get_holdings")
            .with_arg(arg)
            .call()
            .await
            .unwrap();
        let res: Vec<Holding> = candid::Decode!(&bytes, Vec<Holding>).unwrap();
        assert_eq!(res.len(), 3);
    }
}
