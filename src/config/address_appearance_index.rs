use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::parameters::address_appearance_index::{
    DEFAULT_BYTES_PER_ADDRESS, MAX_NETWORK_NAME_BYTES,
};
/// An enum that represents a network as either Mainnet or Other.
///
/// Allows configuration to be changed for different networks as needed.
///
/// Contains network name and number of bytes for each address.
/// # Example
/// ## Mainnet
/// Addresses are 20 bytes long.
/// ```
/// use min_know::config::address_appearance_index::Network;
///
/// let network = Network::default();
/// // Equivalent to:
/// let network = Network::new(20, String::from("mainnet"));
/// ```
/// ## Goerli network
/// Note that only ASCII characters are allowed for the name, otherwise
/// and error will be returned.
/// ```
/// use min_know::config::address_appearance_index::Network;
///
/// let bytes_per_address = 20;
/// let network_name = String::from("goerli");
/// let network = Network::new(bytes_per_address, network_name);
/// ```
#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Deserialize, Serialize)]
pub enum Network {
    Mainnet(Params),
    Other(Params),
}

impl Default for Network {
    /// Default network is mainnet.
    fn default() -> Self {
        Network::Mainnet(Params {
            bytes_per_address: DEFAULT_BYTES_PER_ADDRESS,
            network_name: String::from("mainnet"),
        })
    }
}

impl Network {
    /// Creates a new network config. Checks parameters.
    pub fn new(bytes_per_address: u32, network_name: String) -> Result<Self> {
        if network_name.as_bytes().len() as u32 > MAX_NETWORK_NAME_BYTES || !network_name.is_ascii()
        {
            return Err(anyhow!(
                "The network name must be {} valid ASCII chars or less",
                MAX_NETWORK_NAME_BYTES
            ));
        }
        let params = Network::Other(Params {
            bytes_per_address,
            network_name,
        });
        Ok(params)
    }
    /// Returns the name of the network.
    pub(crate) fn name(&self) -> &str {
        match &self {
            Network::Mainnet(x) => &x.network_name,
            Network::Other(x) => &x.network_name,
        }
    }
}

/// Holds information that may differ between networks. Allows
/// default values to be altered.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Params {
    pub bytes_per_address: u32,
    pub network_name: String,
}
