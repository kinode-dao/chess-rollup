import { useState } from "react";
import { useWeb3React } from "@web3-react/core";
import { ConnectionOptions } from '../libs/components/ConnectionOptions'
import { ConnectionType } from '../libs/connections'
import { BigNumber } from 'ethers'
import useSequencerStore from "../store";

const NavBar = () => {
    let { account, isActive } = useWeb3React();
    let { balances } = useSequencerStore();
    const [connectionType, setConnectionType] = useState<ConnectionType | null>(null)

    return (
        <nav className="flex place-items-center place-content-center grow mx-10">
            <div className="flex justify-between grow">
                <div className="py-5 px-3 grow flex place-items-center">
                    <h1 className="display">
                        Kinode
                        <span className="text-xs self-end">&reg;</span>
                    </h1>
                    <h1 className="mx-auto">
                        ZK Chess
                    </h1>
                </div>
                <div className="py-5 px-3">
                    <code>{account}</code>
                </div>
                {
                    account && (
                        <div className="py-5 px-3">
                            <code>{balances[account.toLowerCase()] && `${BigNumber.from(balances[account.toLowerCase()])} WEI`}</code>
                        </div>
                    )
                }
                <div className="py-5 px-3 flex items-center">
                    <ConnectionOptions
                        activeConnectionType={connectionType}
                        isConnectionActive={isActive}
                        onActivate={setConnectionType}
                        onDeactivate={setConnectionType}
                    />
                </div>
            </div>
        </nav>
    );
};

export default NavBar;