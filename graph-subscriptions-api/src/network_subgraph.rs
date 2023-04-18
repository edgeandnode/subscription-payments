use std::{collections::HashMap, sync::Arc, time::Duration};

use eventuals::{Eventual, EventualExt as _, EventualWriter, Ptr};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use toolshed::bytes::{Address, DeploymentId, SubgraphId};

use crate::subgraph_client;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GraphAccount {
    pub id: Address,
    pub image: Option<String>,
    pub default_display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubgraphVersion {
    pub subgraph: Subgraph,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Subgraph {
    pub id: SubgraphId,
    pub owner: GraphAccount,
    pub display_name: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SubgraphDeployment {
    #[serde(rename = "ipfsHash")]
    id: DeploymentId,
    versions: Vec<SubgraphVersion>,
}

#[derive(Clone)]
pub struct SubgraphDeployments {
    pub inputs: Eventual<Ptr<SubgraphDeploymentInputs>>,
}

#[derive(Clone)]
pub struct SubgraphDeploymentInputs {
    // A DeploymentId is the Qm hash representation of the Subgraph manifest uploaded to decentralized storage (currently IPFS).
    // A SubgraphId is a hash of the owning user address and an incrementing integer owned by the GNS contract.
    // It is possible that multiple users could create the same Subgraph manifest, and therefore get the same Qm hash DeploymentId.
    // And then these multiple users could publish the Subgraph.
    // This creates a scenario where a single DeploymentId could be linked with multiple SubgraphIDs.
    pub deployment_to_subgraphs: HashMap<DeploymentId, Vec<Subgraph>>,
}

impl SubgraphDeployments {
    pub async fn deployment_subgraphs(&self, deployment: &DeploymentId) -> Vec<Subgraph> {
        let map = match self.inputs.value().await {
            std::result::Result::Ok(map) => map,
            Err(_) => return vec![],
        };
        map.deployment_to_subgraphs
            .get(deployment)
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Clone)]
pub struct Data {
    pub subgraph_deployments: SubgraphDeployments,
}

pub struct Client {
    subgraph_client: subgraph_client::Client,
    subgraph_deployments: EventualWriter<Ptr<SubgraphDeploymentInputs>>,
}

impl Client {
    pub fn create(subgraph_client: subgraph_client::Client) -> Data {
        let (subgraph_deployments_tx, subgraph_deployments_rx) = Eventual::new();
        let client = Arc::new(Mutex::new(Client {
            subgraph_client,
            subgraph_deployments: subgraph_deployments_tx,
        }));
        eventuals::timer(Duration::from_secs(30))
            .pipe_async(move |_| {
                let client = client.clone();
                async move {
                    let mut client = client.lock().await;
                    if let Err(poll_subgraphs_err) = client.poll_subgraphs().await {
                        tracing::error!(%poll_subgraphs_err);
                    }
                }
            })
            .forever();

        Data {
            subgraph_deployments: SubgraphDeployments {
                inputs: subgraph_deployments_rx,
            },
        }
    }

    async fn poll_subgraphs(&mut self) -> Result<(), String> {
        let response = self
            .subgraph_client
            .paginated_query::<SubgraphDeployment>(
                r#"
                subgraphDeployments(
                    block: $block
                    orderBy: id, orderDirection: asc
                    first: $first
                    where: {
                        id_gt: $last
                    }
                ) {
                    id
                    ipfsHash
                    versions(
                      orderBy: version
                      orderDirection: asc
                      where: {subgraph_: {active: true, entityVersion: 2}}
                    ) {
                        subgraph {
                            id
                            owner {
                              id
                              image
                              defaultDisplayName
                            }
                            displayName
                            image
                        }
                    }
                }
              "#,
            )
            .await?;
        if response.is_empty() {
            return Err("Discarding empty update (subgraph_deployments)".to_string());
        }
        let deployment_to_subgraphs = parse_deployment_subgraphs(response);

        self.subgraph_deployments
            .write(Ptr::new(SubgraphDeploymentInputs {
                deployment_to_subgraphs,
            }));
        Result::Ok(())
    }
}

fn parse_deployment_subgraphs(
    subgraph_deployment_response: Vec<SubgraphDeployment>,
) -> HashMap<DeploymentId, Vec<Subgraph>> {
    subgraph_deployment_response
        .into_iter()
        .map(|deployment| {
            let subgraphs = deployment
                .versions
                .into_iter()
                .map(|version| version.subgraph)
                .collect();
            (deployment.id, subgraphs)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use anyhow::ensure;
    use serde_json::json;

    use super::*;

    #[test]
    fn should_parse_subgraph_deployment_gql_response_json() -> anyhow::Result<()> {
        let result = serde_json::from_str::<Vec<SubgraphDeployment>>(
            &json!([
              {
                "id": "0x0527631b847f976a3566651d595f5c27c9a13ca464cc8dbcf645bd19365b5b91",
                "ipfsHash": "QmNgmaip92JYzB7RAntXRox3ZcdSjPLHtwYbt94hKeuMxU",
                "versions": [
                  {
                    "subgraph": {
                      "id": "BvSx64tyYGgFY5deaiMVz2sPJrBoo63Bb8htVvqo2GbD",
                      "owner": {
                        "id": "0x8fbbc98259a4ed6e6d6e413c553cc47530e79be8",
                        "image": null,
                        "defaultDisplayName": null
                      },
                      "displayName": "Numero Uno",
                      "image": "https://api.thegraph.com/ipfs/api/v0/cat?arg=QmdSeSQ3APFjLktQY3aNVu3M5QXPfE9ZRK5LqgghRgB7L9"
                    }
                  }
                ]
              }
            ])
            .to_string(),
        );
        ensure!(result.is_ok(), "failed to parse example: {:?}", result);
        Ok(())
    }

    #[test]
    fn should_parse_result_into_map_of_deployment_to_subgraphs() {
        let result = serde_json::from_str::<Vec<SubgraphDeployment>>(
            &json!([
              {
                "id": "0x0527631b847f976a3566651d595f5c27c9a13ca464cc8dbcf645bd19365b5b91",
                "ipfsHash": "QmNgmaip92JYzB7RAntXRox3ZcdSjPLHtwYbt94hKeuMxU",
                "versions": [
                  {
                    "subgraph": {
                      "id": "BvSx64tyYGgFY5deaiMVz2sPJrBoo63Bb8htVvqo2GbD",
                      "owner": {
                        "id": "0x8fbbc98259a4ed6e6d6e413c553cc47530e79be8",
                        "image": null,
                        "defaultDisplayName": null
                      },
                      "displayName": "Numero Uno",
                      "image": "https://api.thegraph.com/ipfs/api/v0/cat?arg=QmdSeSQ3APFjLktQY3aNVu3M5QXPfE9ZRK5LqgghRgB7L9"
                    }
                  }
                ]
              }
            ])
            .to_string(),
        );
        assert!(result.is_ok(), "failed to parse example: {:?}", result);

        let mut expected: HashMap<DeploymentId, Vec<Subgraph>> = HashMap::new();
        let deployment_id =
            DeploymentId::from_ipfs_hash("QmNgmaip92JYzB7RAntXRox3ZcdSjPLHtwYbt94hKeuMxU").unwrap();
        expected.insert(
            deployment_id,
            vec![Subgraph {
              id: "BvSx64tyYGgFY5deaiMVz2sPJrBoo63Bb8htVvqo2GbD".parse::<SubgraphId>().unwrap(),
              display_name: Some(String::from("Numero Uno")),
              image: Some(String::from("https://api.thegraph.com/ipfs/api/v0/cat?arg=QmdSeSQ3APFjLktQY3aNVu3M5QXPfE9ZRK5LqgghRgB7L9")),
              owner: GraphAccount {
                id: "0x8fbbc98259a4ed6e6d6e413c553cc47530e79be8".parse::<Address>().unwrap(),
                image: None,
                default_display_name: None
              }
          }],
        );

        let actual = parse_deployment_subgraphs(result.unwrap());

        assert_eq!(actual.get(&deployment_id), expected.get(&deployment_id));
    }
}
