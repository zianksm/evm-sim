import evm_sim
import web3
import web3.utils


def test_txs():
    url = "http://localhost:8545"

    provider = web3.HTTPProvider(url)

    user1 = "f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    user2 = "9D39A5DE30e57443BfF2A8307A4256c8797A3497"

    simulator = evm_sim.EvmSimulator(url)
    print(str(1e18))
    
    tx = evm_sim.Transaction(user1, user2, str(1), [])
    txs = evm_sim.Txs([tx])
    
    result = simulator.simulate(txs)
