#[cfg(test)]
mod tests {
    use blockxpand_icp::get_holdings;
    use candid::Principal;

    #[tokio::test]
    async fn integration_get_holdings() {
        let principal = Principal::from_text("aaaaa-aa").unwrap();
        let holdings = get_holdings(principal).await;
        assert!(!holdings.is_empty());
    }
}
