#[cfg(test)]
mod tests {
    use blockxpand_icp::get_holdings;
    use candid::Principal;
    use std::process::Command;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn integration_get_holdings() {
        // Skip the test if `dfx` is not installed.
        if Command::new("dfx").arg("--version").output().is_err() {
            eprintln!("dfx not found; skipping integration test");
            return;
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
    }
}
