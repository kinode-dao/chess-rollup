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
        <nav>
            <div className="max-w-6xl mx-auto px-4">
                <div className="flex justify-between">
                    <div className="flex space-x-4">
                        <div className="py-5 px-3">
                            Kinode ZK Chess
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
                        <div className="py-5 px-3">
                            <ConnectionOptions
                                activeConnectionType={connectionType}
                                isConnectionActive={isActive}
                                onActivate={setConnectionType}
                                onDeactivate={setConnectionType}
                            />
                        </div>
                    </div>

                </div>
            </div>
        </nav>
    );
};

export default NavBar;