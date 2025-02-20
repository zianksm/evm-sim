use std::{collections::HashMap, sync::Arc};

use sim::{
    evm_primitives::{self},
    simulator::ExecutionResult,
    utils::Utils,
};
use simulator::simulator_server::SimulatorServer;
use tokio::sync::RwLock;
use tonic::{server, transport::Server, Request, Response, Status};

pub mod simulator {
    tonic::include_proto!("simulator");
}

pub type ChainId = u32;
pub struct Simulators {
    servers: RwLock<HashMap<ChainId, sim::simulator::Simulator>>,
}

impl Simulators {
    pub fn new() -> Self {
        Self {
            servers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn initialize(&self, chain_id: ChainId, url: &str) -> Result<(), Status> {
        let mut servers = self.servers.write().await;

        if servers.contains_key(&chain_id) {
            return Err(Status::already_exists("Chain already initialized"));
        }

        let sim =
            sim::simulator::Simulator::new(url).map_err(|e| Status::internal(e.to_string()))?;

        servers.insert(chain_id, sim);

        Ok(())
    }
}

impl From<ExecutionResult> for simulator::Result {
    fn from(value: ExecutionResult) -> Self {
        match value {
            ExecutionResult::Success {
                reason,
                gas_used,
                gas_refunded,
                logs,
                output,
            } => Self {
                is_success: true,
                revert: None,
                success: Some(simulator::Success {
                    reason: serde_json::to_string_pretty(&reason)
                        .expect("internal serialization should not fail"),
                    gas_used,
                    gas_refunded,
                    logs: logs.into_iter().map(Into::into).collect(),
                    output: Some(output.into()),
                }),
            },
            ExecutionResult::Revert { gas_used, output } => todo!(),
            ExecutionResult::Halt { reason, gas_used } => todo!(),
        }
    }
}

impl From<evm_primitives::Log> for simulator::Log {
    fn from(value: evm_primitives::Log) -> Self {
        let topics = value.topics();
        let data = &value.data.data;
        let address = value.address.to_string();

        Self {
            address,
            topics: topics.iter().map(hex::encode).collect(),
            data: hex::encode(data),
        }
    }
}

impl From<evm_primitives::Output> for simulator::Output {
    fn from(value: evm_primitives::Output) -> Self {
        let address = value.address().map(|v| v.to_string());
        let data = hex::encode(value.into_data());

        Self {
            is_call: address.is_some(),
            hex_data: data,
            address,
        }
    }
}

#[tonic::async_trait]
impl simulator::simulator_server::Simulator for Simulators {
    async fn simulate(
        &self,
        request: Request<simulator::Transactions>,
    ) -> Result<Response<simulator::Results>, Status> {
        let req = request.into_inner();
        let chain_id = req.chain_id;
        let txs = req
            .transactions
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let servers = self.servers.read().await;

        let sim = servers
            .get(&chain_id)
            .ok_or_else(|| Status::not_found("No simulator found"))?;

        let results = sim.simulate(txs);

        for result in results {
            match result {
                Ok(result) => {}
                Err(e) => {
                    return Err(Status::internal(e.to_string()));
                }
            }
        }

        todo!()
    }

    // TODO: add caches for urls
    async fn initialize(
        &self,
        request: Request<simulator::InitializeRequest>,
    ) -> Result<Response<simulator::InitializeReply>, Status> {
        let mut servers = self.servers.write().await;
        let req = request.into_inner();

        if servers.contains_key(&req.chain_id) {
            return Err(Status::already_exists("Chain already initialized"));
        }

        let sim = sim::simulator::Simulator::new(&req.url)
            .map_err(|e| Status::internal(e.to_string()))?;

        servers.insert(req.chain_id, sim);

        Ok(Response::new(simulator::InitializeReply {
            message: "Initialized".into(),
        }))
    }
}

impl TryFrom<simulator::Transaction> for sim::evm::Tx {
    type Error = tonic::Status;

    fn try_from(value: simulator::Transaction) -> Result<Self, Self::Error> {
        Ok(Self {
            data: hex::decode(value.hex_data)
                .map_err(|_| Status::invalid_argument("Invalid data"))?,
            from: Utils::address_try_from_string(value.from)
                .map_err(|_| Status::invalid_argument("Invalid from"))?,
            to: Utils::address_try_from_string(value.to)
                .map_err(|_| Status::invalid_argument("Invalid to"))?,
            value: Utils::uint_try_from_string(&value.value)
                .map_err(|_| Status::invalid_argument("Invalid value"))?,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    let sim = Simulators::new();

    Server::builder()
        .add_service(SimulatorServer::new(sim))
        .serve(addr)
        .await?;

    Ok(())
}
