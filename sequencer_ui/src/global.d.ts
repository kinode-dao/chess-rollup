interface Window {
    ethereum?: {
        request: (request: { method: string, params?: Array<any> }) => Promise<any>;
        // Add other Ethereum methods and properties you need here
    };
}