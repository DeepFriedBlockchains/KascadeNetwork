use clap::{CommandFactory, Parser, error::ErrorKind};
use futures::future::join_all;
use kaspa_grpc_client::GrpcClient;
use kaspa_rpc_core::{RpcHash, RpcNetworkId, RpcNetworkType, api::rpc::RpcApi};
use serde::Serialize;
use std::{collections::HashSet, net::SocketAddr, process::ExitCode, time::Duration};

#[derive(Debug, Parser)]
#[command(about = "Check health and chain convergence of local Simnet gRPC nodes")]
struct Args {
    /// Loopback gRPC addresses as IP:port. Enclose IPv6 addresses in brackets.
    #[arg(required = true, num_args = 1..=8, value_parser = parse_loopback)]
    endpoints: Vec<SocketAddr>,
    /// Minimum connected peers required on each node. Use 0 for a standalone node.
    #[arg(long, default_value_t = 1)]
    minimum_peers: usize,
    /// Maximum time in milliseconds for each node's connection and RPC sample.
    #[arg(long, default_value_t = 3_000, value_parser = clap::value_parser!(u64).range(1..=60_000))]
    timeout_ms: u64,
}

fn parse_loopback(value: &str) -> Result<SocketAddr, String> {
    let endpoint: SocketAddr = value.parse().map_err(|_| "expected a numeric IP address and port".to_owned())?;
    if !endpoint.ip().is_loopback() || endpoint.port() == 0 {
        return Err("expected a loopback IP address and a nonzero port".to_owned());
    }
    Ok(endpoint)
}

#[derive(Debug, Serialize)]
struct NodeState {
    network: RpcNetworkId,
    version: String,
    is_synced: bool,
    connected_peers: usize,
    block_count: u64,
    header_count: u64,
    selected_tip: RpcHash,
    pruning_point: RpcHash,
    virtual_daa_score: u64,
}

impl NodeState {
    fn agrees_with(&self, other: &Self) -> bool {
        self.network == other.network
            && self.selected_tip == other.selected_tip
            && self.pruning_point == other.pruning_point
            && self.virtual_daa_score == other.virtual_daa_score
    }
}

#[derive(Debug, Serialize)]
struct NodeReport {
    endpoint: SocketAddr,
    #[serde(flatten)]
    outcome: NodeOutcome,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum NodeOutcome {
    Sampled { state: NodeState },
    Failed { reason: String },
}

#[derive(Debug, Serialize)]
struct ProbeReport {
    healthy: bool,
    converged: bool,
    minimum_peers: usize,
    nodes: Vec<NodeReport>,
    issues: Vec<String>,
}

fn summarize(nodes: Vec<NodeReport>, minimum_peers: usize) -> ProbeReport {
    let reference = nodes.iter().find_map(|node| match &node.outcome {
        NodeOutcome::Sampled { state } => Some((node.endpoint, state)),
        NodeOutcome::Failed { .. } => None,
    });
    let mut converged = reference.is_some();
    let mut issues = Vec::new();
    for node in &nodes {
        let endpoint = node.endpoint;
        match &node.outcome {
            NodeOutcome::Failed { reason } => {
                issues.push(format!("{endpoint}: {reason}"));
                converged = false;
            }
            NodeOutcome::Sampled { state } => {
                if state.connected_peers < minimum_peers {
                    issues.push(format!("{endpoint}: {} peers, expected at least {minimum_peers}", state.connected_peers));
                }
                if let Some((reference_endpoint, reference_state)) = reference
                    && !state.agrees_with(reference_state)
                {
                    issues.push(format!(
                        "{endpoint}: network, selected tip, pruning point or virtual DAA score differs from {reference_endpoint}"
                    ));
                    converged = false;
                }
            }
        }
    }
    ProbeReport { healthy: converged && issues.is_empty(), converged, minimum_peers, nodes, issues }
}

async fn sample_node(endpoint: SocketAddr) -> Result<NodeState, String> {
    let client = GrpcClient::connect(format!("grpc://{endpoint}")).await.map_err(|err| format!("gRPC connection failed: {err}"))?;
    let observation = async {
        let server = client.get_server_info().await.map_err(|err| format!("GetServerInfo failed: {err}"))?;
        if server.network_id.network_type() != RpcNetworkType::Simnet {
            return Err(format!("expected Simnet, found {}", server.network_id));
        }
        let (dag, peers) = tokio::try_join!(client.get_block_dag_info(), client.get_connected_peer_info())
            .map_err(|err| format!("DAG or peer query failed: {err}"))?;
        if dag.network != server.network_id {
            return Err(format!("inconsistent network identity: {} and {}", server.network_id, dag.network));
        }
        Ok(NodeState {
            network: dag.network,
            version: server.server_version,
            is_synced: server.is_synced,
            connected_peers: peers.peer_info.len(),
            block_count: dag.block_count,
            header_count: dag.header_count,
            selected_tip: dag.sink,
            pruning_point: dag.pruning_point_hash,
            virtual_daa_score: dag.virtual_daa_score,
        })
    }
    .await;
    let _ = client.disconnect().await;
    observation
}

async fn probe_node(endpoint: SocketAddr, timeout: Duration) -> NodeReport {
    let outcome = match tokio::time::timeout(timeout, sample_node(endpoint)).await {
        Ok(Ok(state)) => NodeOutcome::Sampled { state },
        Ok(Err(reason)) => NodeOutcome::Failed { reason },
        Err(_) => NodeOutcome::Failed { reason: format!("gRPC sample timed out after {} ms", timeout.as_millis()) },
    };
    NodeReport { endpoint, outcome }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args = Args::parse();
    if args.endpoints.iter().collect::<HashSet<_>>().len() != args.endpoints.len() {
        Args::command().error(ErrorKind::ValueValidation, "each endpoint must be unique").exit();
    }
    let timeout = Duration::from_millis(args.timeout_ms);
    let nodes = join_all(args.endpoints.into_iter().map(|endpoint| probe_node(endpoint, timeout))).await;
    let report = summarize(nodes, args.minimum_peers);
    println!("{}", serde_json::to_string_pretty(&report).expect("health report is JSON serializable"));
    if report.healthy { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(port: u16, peers: usize) -> NodeReport {
        NodeReport {
            endpoint: ([127, 0, 0, 1], port).into(),
            outcome: NodeOutcome::Sampled {
                state: NodeState {
                    network: RpcNetworkId::new(RpcNetworkType::Simnet),
                    version: "test".to_owned(),
                    is_synced: true,
                    connected_peers: peers,
                    block_count: 3,
                    header_count: 3,
                    selected_tip: RpcHash::from_bytes([1; 32]),
                    pruning_point: RpcHash::from_bytes([2; 32]),
                    virtual_daa_score: 3,
                },
            },
        }
    }

    #[test]
    fn accepts_only_numeric_loopback_endpoints_and_bounded_timeout() {
        for endpoint in ["127.0.0.1:19110", "[::1]:19110"] {
            assert!(Args::try_parse_from(["probe", endpoint]).is_ok());
        }
        for endpoint in ["localhost:19110", "0.0.0.0:19110", "192.0.2.1:19110", "127.0.0.1:0"] {
            assert!(Args::try_parse_from(["probe", endpoint]).is_err());
        }
        assert!(Args::try_parse_from(["probe", "--timeout-ms", "0", "127.0.0.1:19110"]).is_err());
        let too_many = std::iter::once("probe".to_owned()).chain((1..=9).map(|port| format!("127.0.0.1:{port}")));
        assert!(Args::try_parse_from(too_many).is_err());
    }

    #[test]
    fn reports_both_agreement_and_peer_health() {
        let healthy = summarize(vec![sample(19110, 2), sample(19210, 1)], 1);
        assert!(healthy.healthy && healthy.converged);
        let json = serde_json::to_value(&healthy).unwrap();
        assert_eq!(json["nodes"][0]["state"]["network"], "simnet");
        let disconnected = summarize(vec![sample(19110, 2), sample(19210, 0)], 1);
        assert!(!disconnected.healthy && disconnected.converged);
        assert_eq!(disconnected.issues.len(), 1);
        assert!(summarize(vec![sample(19110, 0)], 0).healthy);
    }

    #[test]
    fn detects_divergent_chain_state_and_unavailable_nodes() {
        for change in
            [(Some(RpcHash::from_bytes([3; 32])), None, None), (None, Some(RpcHash::from_bytes([4; 32])), None), (None, None, Some(4))]
        {
            let mut divergent = sample(19210, 1);
            let NodeOutcome::Sampled { state } = &mut divergent.outcome else { unreachable!() };
            state.selected_tip = change.0.unwrap_or(state.selected_tip);
            state.pruning_point = change.1.unwrap_or(state.pruning_point);
            state.virtual_daa_score = change.2.unwrap_or(state.virtual_daa_score);
            let report = summarize(vec![sample(19110, 2), divergent], 1);
            assert!(!report.healthy && !report.converged);
        }
        let unavailable =
            NodeReport { endpoint: ([127, 0, 0, 1], 19210).into(), outcome: NodeOutcome::Failed { reason: "unreachable".into() } };
        let report = summarize(vec![sample(19110, 2), unavailable], 1);
        assert!(!report.healthy && !report.converged);
        assert!(report.issues[0].contains("unreachable"));
    }

    #[tokio::test]
    async fn times_out_when_a_loopback_listener_never_answers_rpc() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap();
        let report = probe_node(endpoint, Duration::from_millis(30)).await;
        let NodeOutcome::Failed { reason } = report.outcome else { panic!("silent listener yielded an observation") };
        assert!(reason.contains("timed out"), "{reason}");
    }
}
