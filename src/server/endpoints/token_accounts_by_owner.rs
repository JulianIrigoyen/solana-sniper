//
//
// async fn query_token_accounts_by_owner(whale_pubkey: String) -> Result<Vec<TokenAccount>, Box<dyn Error>> {
//     // Setup the RPC request similar to the previous examples
//     let rpc_request = json!({
//         "jsonrpc": "2.0",
//         "id": 1,
//         "method": "getTokenAccountsByOwner",
//         "params": [
//             whale_pubkey,
//             {"mint": "<TOKEN_MINT_ADDRESS>"},
//             {"encoding": "jsonParsed"}
//         ]
//     });
//
//     // Send the request to the Solana RPC endpoint and handle the response
//     // Assuming a send_rpc_request is a utility function you've implemented
//     let response = send_rpc_request(rpc_request).await?;
//
//     // Process the response to extract token account details
//     // Further processing to be implemented based on your needs
//     Ok(response)
// }
