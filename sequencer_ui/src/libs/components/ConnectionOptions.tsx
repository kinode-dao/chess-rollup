import React from 'react'

import { ConnectionType, getHasMetaMaskExtensionInstalled } from '../connections'
import { METAMASK_URL } from '../constants'
import { Option } from './Option'

type ConnectOptionsParams = {
    activeConnectionType: ConnectionType | null
    isConnectionActive: boolean
    onActivate: (connectionType: ConnectionType) => void
    onDeactivate: (connectionType: null) => void
}

export const ConnectionOptions = ({
    activeConnectionType,
    isConnectionActive,
    onActivate,
    onDeactivate,
}: ConnectOptionsParams) => {
    function getOptions(isActive: boolean) {
        const hasMetaMaskExtension = getHasMetaMaskExtensionInstalled()

        const isNoOptionActive = !isActive || (isActive && activeConnectionType === null)

        const metaMaskOption = hasMetaMaskExtension ? (
            <Option
                isEnabled={isNoOptionActive || activeConnectionType === ConnectionType.INJECTED}
                isConnected={activeConnectionType === ConnectionType.INJECTED}
                connectionType={ConnectionType.INJECTED}
                onActivate={onActivate}
                onDeactivate={onDeactivate}
            />
        ) : (
            <a href={METAMASK_URL}>
                <button>Install Metamask</button>
            </a>
        )

        return (
            <>
                {metaMaskOption}
            </>
        )
    }

    return <div className="connectors">{getOptions(isConnectionActive)}</div>
}