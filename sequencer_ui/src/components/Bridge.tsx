import { useState, useCallback, FormEvent } from "react";
import { ethers } from "ethers";
import { useWeb3React } from "@web3-react/core";
import { BigNumber } from 'ethers'
import ROLLUP_ABI from "../abis/bridge.json";
import { SEPOLIA_CHAIN_ID } from "../libs/constants";

const Bridge = () => {
    let { account, provider, chainId } = useWeb3React();
    const [amount, setAmount] = useState(0);

    const bridge = useCallback(
        async (e: FormEvent) => {
            e.preventDefault();
            try {
                if (!account || !provider) {
                    window.alert('Ethereum wallet is not connected');
                    return;
                }
                if (chainId !== SEPOLIA_CHAIN_ID) {
                    window.alert('Please connect to sepolia');
                    return;
                }

                const rollup = new ethers.Contract('0xA25489Af7c695DE69eDd19F7A688B2195B363f23', ROLLUP_ABI, provider.getSigner());
                const receipt = await rollup.deposit({ value: BigNumber.from(amount) });

                console.log('receipt', receipt);
            } catch (err) {
                console.error(err);
            }
        },
        [account, provider, amount, setAmount]
    );


    return (
        <div>
            <h4 className="m-2">Bridge ETH (value in WEI) from Sepolia</h4>
            <div className="flex flex-col overflow-scroll">
                <form onSubmit={bridge}>
                    <input
                        type="text"
                        value={amount}
                        onChange={(e) => setAmount(Number(e.target.value))}
                    />
                    <button type="submit">Bridge</button>
                </form>
            </div>
        </div>
    );
};

export default Bridge;