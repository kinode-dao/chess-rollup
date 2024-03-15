import { useState, useCallback, FormEvent } from "react";
import { ethers } from "ethers";
import { useWeb3React } from "@web3-react/core";
import { BigNumber } from 'ethers'
import ROLLUP_ABI from "../abis/rollup.json";
import { SEPOLIA_CHAIN_ID } from "../libs/constants";

const Bridge = () => {
    let { account, provider, chainId } = useWeb3React();
    const [amount, setAmount] = useState(0);

    const proposeGame = useCallback(
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

                const rollup = new ethers.Contract('0x8b2fbb3f09123e478b55209ec533f56d6ee83b8b', ROLLUP_ABI, provider.getSigner());
                const receipt = await rollup.depositEth(0, account, { value: BigNumber.from(amount) });

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
                <form onSubmit={proposeGame}>
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