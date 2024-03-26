import { useCallback } from "react";
import { ethers } from "ethers";
import { useWeb3React } from "@web3-react/core";
import ROLLUP_ABI from "../abis/Bridge.json";
import { BRIDGE_ADDRESS } from "../libs/constants";
import useSequencerStore, { Claim } from "../store";

const Bridge = () => {
    let { account, provider, chainId } = useWeb3React();
    const { batches } = useSequencerStore();

    const withdraw = useCallback(
        async (batchIndex: string, index: number, amount: string, proof: string[]) => {
            try {
                if (!account || !provider || !chainId) {
                    window.alert('Ethereum wallet is not connected');
                    return;
                }
                if (!BRIDGE_ADDRESS[chainId]) {
                    window.alert('Please connect to Optimism mainnet');
                    return;
                }

                const rollup = new ethers.Contract(BRIDGE_ADDRESS[chainId], ROLLUP_ABI, provider.getSigner());
                const receipt = await rollup.withdraw(batchIndex, index, account, amount, proof);

                console.log('receipt', receipt);
            } catch (err) {
                console.error(err);
            }
        },
        [account, provider, chainId]
    );

    if (!account) return <></>

    let accountClaims: Record<string, Claim> = {};
    // TODO this is super inefficient
    batches.forEach((batch, index) => {
        Object.keys(batch.claims).forEach((key) => {
            if (key === account!.toLowerCase()) {
                accountClaims[index] = batch.claims[key];
            }
        });
    })
    console.log(accountClaims)
    if (!accountClaims) return <></>

    return (
        <>
            {
                Object.keys(accountClaims).map((key, i) => {
                    const { index, amount, proof } = accountClaims[key];
                    return (
                        <button key={i} onClick={(_) => withdraw(key, index, amount, proof)}>{`Withdraw ${amount} tokens`}</button>
                    )
                })
            }
        </>
    );
};

export default Bridge;