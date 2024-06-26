interface Window {
    ethereum?: {
        // value that is populated and returns true by the Coinbase Wallet mobile dapp browser
        isCoinbaseWallet?: true
        isMetaMask?: true
        autoRefreshOnNetworkChange?: boolean
        isBraveWallet?: true
        request: (request: { method: string, params?: Array<any> }) => Promise<any>;
    }
}