// Sets if the example should run locally or on chain
export enum Chain {
    SEPOLIA,
    MAINNET,
}

// Inputs that configure this example to run
interface ExampleConfig {
    chain: Chain
    rpc: {
        sepolia: string
        mainnet: string
        optimism: string
    }
}

// Example Configuration
export const CurrentConfig: ExampleConfig = {
    chain: Chain.MAINNET,
    rpc: {
        sepolia: 'https://sepolia.infura.io/v3/',
        mainnet: 'https://mainnet.infura.io/v3/',
        optimism: 'https://mainnet.optimism.io/v1/',
    },
}