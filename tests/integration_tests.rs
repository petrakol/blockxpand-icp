#[cfg(test)]
mod tests {
    use blockxpand_icp_aggregator::aggregator;
    use candid::Principal;

    #[tokio::test]
    async fn integration_get_holdings() {
        let principal = Principal::from_text("aaaaa-aa").unwrap();
        let holdings = aggregator::get_holdings(principal).await;
        assert_eq!(holdings.len(), 4);
    }
}
