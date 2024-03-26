import { Chain, CurrentConfig } from '../config'

// Chains
export const MAINNET_CHAIN_ID = 1
export const OPTIMISM_CHAIN_ID = 10
export const SEPOLIA_CHAIN_ID = 11155111

export const INPUT_CHAIN_ID = CurrentConfig.chain === Chain.SEPOLIA ? SEPOLIA_CHAIN_ID : MAINNET_CHAIN_ID
export const INPUT_CHAIN_URL =
    CurrentConfig.chain === Chain.SEPOLIA ? CurrentConfig.rpc.sepolia : CurrentConfig.rpc.mainnet

export const CHAIN_TO_URL_MAP = {
    [SEPOLIA_CHAIN_ID]: CurrentConfig.rpc.sepolia,
    [MAINNET_CHAIN_ID]: CurrentConfig.rpc.mainnet,
    [OPTIMISM_CHAIN_ID]: CurrentConfig.rpc.optimism,
}

type ChainInfo = {
    explorer: string
    label: string
    nativeCurrency: {
        name: string
        symbol: string
        decimals: 18
    }
    rpcUrl: string
}

export const CHAIN_INFO: { [key: string]: ChainInfo } = {
    [MAINNET_CHAIN_ID]: {
        explorer: 'https://etherscan.io/',
        label: 'Ethereum',
        nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
        rpcUrl: CurrentConfig.rpc.mainnet,
    },
    [SEPOLIA_CHAIN_ID]: {
        explorer: '',
        label: 'Ethereum',
        nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
        rpcUrl: CurrentConfig.rpc.sepolia,
    },
    [OPTIMISM_CHAIN_ID]: {
        explorer: '',
        label: 'Optimism',
        nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
        rpcUrl: CurrentConfig.rpc.optimism,
    },
}

// URLs
export const METAMASK_URL = 'https://metamask.io/'

interface ContractMap {
    [chainId: number]: string
}

export const BRIDGE_ADDRESS: ContractMap = {
    // [SEPOLIA_CHAIN_ID]: "0xA25489Af7c695DE69eDd19F7A688B2195B363f23",
    [OPTIMISM_CHAIN_ID]: "0x24E063a827CB134315aC57A380446c8bF5418555",
}