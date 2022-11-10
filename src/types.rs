//! Types that are used to manage the address-appearance-index.
//!
//! Some structs derive `TreeHash` according to the SSZ spec. Where this
//! is the case, vectors and lists use the ssz_types crate and are of the form:
//!
//! ```ignore
//! type a_list = VariableList<some_type, max_length>;
//! type a_vector = FixedVector<some_type, length>;
//! ```
//! See the address-appearance-index [spec][1] for the required types.
//!
//! [1]: [specification](https://github.com/perama-v/address-appearance-index-specs).
use anyhow::{anyhow, Context};
use directories;

use std::{fs, path::PathBuf};

use crate::{
    constants::{DEFAULT_BYTES_PER_ADDRESS, MAX_NETWORK_NAME_BYTES},
    spec::{ChapterIdentifier, VolumeIdentifier},
    utils::{chapter_dir_name, manifest_version_ok, name_to_num, volume_file_name},
};

/// A path representing the location of data for the Unchained Index.
///
///
/// The Unchained Index (unix platforms only) is stored using standard
/// naming conventions (outlined below). To access the local data, the `Default` variant
/// will point to the real Unchained Index data. The `Sample` variant will point
/// to the directory where the address-index-data is kept.
///
/// # Example
/// For sample data:
/// ```
/// use min_know::types::UnchainedPath;
///
/// let path = UnchainedPath::Sample;
/// ```
/// For real data, located where the Unchained Index data lives by default:
/// ```
/// use min_know::types::UnchainedPath;
///
/// let path = UnchainedPath::Default;
/// ```
/// For custom data:
/// ```
/// use std::path::PathBuf;
/// use min_know::types::UnchainedPath;
///
/// let my_dir = PathBuf::from("./");
/// let path = UnchainedPath::Custom(my_dir);
/// ```
///
/// # Cross-platform naming convention
///
/// The [directories](https://crates.io/crates/directories) crate is used for cross
/// platform folder naming for the `Default` variant.
/// - Linux: XDG base directory and the XDG user directory specifications.
///     - First: $XDG_DATA_HOME/<project_path>
///     - Second: $HOME/.local/share/<project_path>
/// - Windows: Not supported by TrueBlocks / Unchained Index
///     - Use `UnchainedPath::Custom()` instead.
/// - macOS: Standard Directories guidelines.
///     - $HOME/Library/Application Support/<project_path>
///
/// The Unchained Index real data (`Default` variant) path is constructed using:
/// - qualifier: ""
/// - organization: ""
/// - application: "trueblocks"
///
/// The sample chunks data (`Sample` variant) path is constructed using:
/// - qualifier: ""
/// - organization: ""
/// - application: "address-appearance-index"
///
/// The `Custom()` variant uses the specified path.
#[derive(Debug, Clone)]
pub enum UnchainedPath {
    Sample,
    Default,
    Custom(PathBuf),
}

impl UnchainedPath {
    /// Gets the path to the chunks directory of the Unchained Index.
    ///
    /// # Example
    /// For sample data:
    /// ```
    /// use min_know::types::{UnchainedPath, Network};
    ///
    /// let path = UnchainedPath::Sample;
    /// let network = Network::default();
    /// let chunks = path.chunks_dir(&network);
    /// // On linux: chunks = ~/.local/share/address-appearance-index/samples/unchained_index_mainnet
    /// ```
    /// For real data:
    /// ```
    /// use min_know::types::{UnchainedPath, Network};
    ///
    /// let path = UnchainedPath::Default;
    /// let network = Network::default();
    /// let chunks = path.chunks_dir(&network);
    /// // On linux: chunks = ~/.local/share/trueblocks/unchained/mainnet/finalized
    /// ```
    ///
    /// # Sample data
    ///
    /// Where the `Network.name()` value is "NETWORK" the `Sample` variant
    /// returns the chunks directory as follows:
    ///
    /// The `Sample` directory follows the naming conventions:
    /// - Linux: XDG base directory and the XDG user directory specifications.
    ///     - First: $XDG_DATA_HOME/address-appearance-index/samples/unchained_index_NETWORK
    ///     - Second: $HOME/.local/share/~/.local/share/address-appearance-index/samples/unchained_index_NETWORK
    /// - Windows: Known Folder API.
    ///     - {FOLDERID_RoamingAppData}/address-appearance-index/data/samples/unchained_index_NETWORK
    /// - macOS: Standard Directories guidelines.
    ///     - $HOME/Library/Application Support/address-appearance-index/samples/unchained_index_NETWORK
    ///
    /// # Default data location
    ///
    /// Where the `Network.name()` value is "NETWORK" the `Default` variant
    /// returns the chunks directory as follows:
    ///
    /// - Linux: XDG base directory and the XDG user directory specifications.
    ///     - First: $XDG_DATA_HOME/trueblocks/unchained/NETWORK/finalized
    ///     - Second: $HOME/.local/share/trueblocks/unchained/NETWORK/finalized
    /// - Windows: Not supported by TrueBlocks / Unchained Index
    ///     - Use `UnchainedPath::Custom()` instead.
    /// - macOS: Standard Directories guidelines.
    ///     - $HOME/Library/Application Support/trueblocks/unchained/NETWORK/finalized
    ///
    /// # Custom data location
    ///
    /// Where the `Network.name()` value is "NETWORK" the `Custom` variant
    /// returns the chunks directory as follows:
    ///
    /// CUSTOM_PATH/samples/unchained_index_NETWORK
    pub fn chunks_dir(&self, network: &Network) -> Result<PathBuf, anyhow::Error> {
        match self {
            UnchainedPath::Custom(p) => {
                // append samples/unchained_index_NETWORK
                let dir_name = format!("unchained_index_{}", network.name());
                Ok(p.join("samples").join(dir_name))
            }
            UnchainedPath::Sample => {
                // use address-dirs and append samples/unchained_index_NETWORK
                let dir_name = format!("unchained_index_{}", network.name());
                let proj = directories::ProjectDirs::from("", "", "address-appearance-index")
                    .ok_or_else(|| {
                        anyhow!("Could not access env var (e.g., $HOME) to set up project.")
                    })?;
                Ok(proj.data_dir().join("samples").join(dir_name))
            }
            UnchainedPath::Default => {
                // use unchained-dirs and append unchained/NETWORK/finalized
                let proj =
                    directories::ProjectDirs::from("", "", "trueblocks").ok_or_else(|| {
                        anyhow!("Could not access env var (e.g., $HOME) to set up project.")
                    })?;
                Ok(proj
                    .data_dir()
                    .join("unchained")
                    .join(network.name())
                    .join("finalized"))
            }
        }
    }
}

/// A path representing the location of data (either source or destination),
/// for the address-appearance-index.
///
/// # Example
/// To specify the samples directory:
/// ```
/// use min_know::types::AddressIndexPath;
///
/// let path = AddressIndexPath::Sample;
/// ```
/// To specify the data to be within the current project directory (will be
/// placed inside a "./data" directory.
/// ```
/// use std::path::PathBuf;
/// use min_know::types::AddressIndexPath;
///
/// let some_directory = PathBuf::from("./");
/// let path = AddressIndexPath::Custom(some_directory);
/// # Ok::<(), anyhow::Error>(())
/// ```
/// The `Default` and `Sample` directories follow the naming conventions:
/// - Linux: XDG base directory and the XDG user directory specifications.
///     - First: $XDG_DATA_HOME/<project_path>
///     - Second: $HOME/.local/share/<project_path>
/// - Windows: Known Folder API.
///     - {FOLDERID_RoamingAppData}/<project_path>/data
/// - macOS: Standard Directories guidelines.
///     - $HOME/Library/Application Support/<project_path>
#[derive(Debug, Clone)]
pub enum AddressIndexPath {
    Sample,
    Default,
    Custom(PathBuf),
}

impl AddressIndexPath {
    /// Returns the directory for the index for the given network.
    ///
    /// This directory will contain the index directory (which contains chapter directories).
    /// Conforms to the `ProjectDirs.data_dir()` schema in the Directories crate.
    /// ## Examples
    ///
    /// ### Sample data
    ///
    /// - Linux: XDG base directory and the XDG user directory specifications.
    ///     - First: $XDG_DATA_HOME/address-appearance-index/samples/address_appearance_index_mainnet
    ///     - Second: $HOME/.local/share/address-appearance-index/samples/address_appearance_index_mainnet
    /// - Windows: Known Folder API.
    ///     - {FOLDERID_RoamingAppData}/address-appearance-index/data/samples/address_appearance_index_mainnet
    /// - macOS: Standard Directories guidelines.
    ///     - $HOME/Library/Application Support/address-appearance-index/samples/address_appearance_index_mainnet
    ///
    /// ### Default data
    ///
    /// - Linux: XDG base directory and the XDG user directory specifications.
    ///     - First: $XDG_DATA_HOME/address-appearance-index/address_appearance_index_mainnet
    ///     - Second: $HOME/.local/share/address-appearance-index/address_appearance_index_mainnet
    /// - Windows: Known Folder API.
    ///     - {FOLDERID_RoamingAppData}/address-appearance-index/data/address_appearance_index_mainnet
    /// - macOS: Standard Directories guidelines.
    ///     - $HOME/Library/Application Support/address-appearance-index/address_appearance_index_mainnet
    pub fn index_dir(&self, network: &Network) -> Result<PathBuf, anyhow::Error> {
        let index_dir_name = format!("address_appearance_index_{}", network.name());
        match directories::ProjectDirs::from("", "", "address-appearance-index") {
            Some(p) => match self {
                AddressIndexPath::Sample => Ok(PathBuf::from(p.data_dir())
                    .join("samples")
                    .join(index_dir_name)),
                AddressIndexPath::Default => Ok(PathBuf::from(p.data_dir()).join(index_dir_name)),
                AddressIndexPath::Custom(root) => Ok(root.to_path_buf().join(index_dir_name)),
            },
            None => Err(anyhow!(
                "Could not access env var (e.g., $HOME) to set up project."
            )),
        }
    }
    /// Returns the directory for the specified chapter.
    ///
    /// # Example
    /// For `mainnet` and chapter "0x4e", returns
    /// "xyz/address_appearance_index_mainnet/chapter_0x4e"
    pub fn chapter_dir(&self, network: &Network, chapter: &str) -> Result<PathBuf, anyhow::Error> {
        let index_dir = self.index_dir(network);
        let chap_name = chapter_dir_name(chapter);
        Ok(index_dir?.join(chap_name))
    }
    /// Returns the path of a given manifest file.
    pub fn manifest_file(&self, network: &Network) -> Result<PathBuf, anyhow::Error> {
        // Use first file starting with "manifest".
        let index_dir = self.index_dir(network)?;
        let manifest = fs::read_dir(&index_dir)
            .with_context(|| format!("Failed to read dir: {:?}", &index_dir))?
            .filter_map(|f| f.ok())
            .filter_map(|f| f.file_name().into_string().ok())
            .find(|f| f.starts_with("manifest"))
            .ok_or_else(|| anyhow!("No manifest file found in: {:?}", &index_dir))?;
        // Before attempting decoding, check the version for compatibility.
        manifest_version_ok(&manifest)?;
        // Read file.
        Ok(index_dir.join(&manifest))
    }
    /// Returns the path of a given volume file.
    pub fn volume_file(
        &self,
        network: &Network,
        chapter: &str,
        first_block: u32,
    ) -> Result<PathBuf, anyhow::Error> {
        let chap_dir = self.chapter_dir(network, chapter)?;
        let filename = volume_file_name(chapter, first_block)?;
        Ok(chap_dir.join(filename))
    }
    /// Returns the latest volume file id present in the index.
    ///
    /// Just checks chapter 0x00 to see what the block height is up to.
    pub fn latest_volume(&self, network: &Network) -> Result<VolumeIdentifier, anyhow::Error> {
        let mut highest: u32 = 0;
        for file in fs::read_dir(self.chapter_dir(network, "00")?)? {
            let name = file?.file_name();
            let num = name_to_num(
                name.to_str()
                    .ok_or_else(|| anyhow!("Chapter contains bad volume file: {:?}", name))?,
            )?;
            if num > highest {
                highest = num
            }
        }
        Ok(VolumeIdentifier {
            oldest_block: highest,
        })
    }
}

/// An enum that represents a network as either Mainnet or Other.
///
/// Allows configuration to be changed for different networks as needed.
///
/// Contains network name and number of bytes for each address.
/// # Example
/// ## Mainnet
/// Addresses are 20 bytes long.
/// ```
/// use min_know::types::Network;
///
/// let network = Network::default();
/// // Equivalent to:
/// let network = Network::new(20, String::from("mainnet"));
/// ```
/// ## Goerli network
/// Note that only ASCII characters are allowed for the name, otherwise
/// and error will be returned.
/// ```
/// use min_know::types::Network;
///
/// let bytes_per_address = 20;
/// let network_name = String::from("goerli");
/// let network = Network::new(bytes_per_address, network_name);
/// ```
#[derive(Debug, Clone)]
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
    pub fn new(bytes_per_address: u32, network_name: String) -> Result<Self, anyhow::Error> {
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
    pub fn name(&self) -> &str {
        match &self {
            Network::Mainnet(x) => &x.network_name,
            Network::Other(x) => &x.network_name,
        }
    }
}

/// Holds information that may differ between networks. Allows
/// default values to be altered.
#[derive(Debug, Clone)]
pub struct Params {
    pub bytes_per_address: u32,
    pub network_name: String,
}

/// An audit helper that holds which volumes an incomplete chapter has/lacks.
///
/// A manifest built on sample data will only expect sample data, and
/// so the `absent` category will be non-exhaustive relative to real
/// data (it only checks that the sample files are present).
#[derive(Debug)]
pub struct ChapterCompleteness {
    /// The identifier of the chapter.
    pub id: ChapterIdentifier,
    /// The volume file is present and has the correct hash.
    pub ok: Vec<VolumeIdentifier>,
    /// The volume file is missing.
    pub absent: Vec<VolumeIdentifier>,
    /// The calculated volume ssz hash tree root is different from manifest.
    pub bad_hash: Vec<VolumeIdentifier>,
}

/// Represents the difference between the manifest and the index data.
///
/// Used to compare local data with what could be obtained from peers.
#[derive(Debug)]
pub struct IndexCompleteness {
    pub complete_chapters: Vec<ChapterIdentifier>,
    pub absent_chapters: Vec<ChapterIdentifier>,
    pub incomplete_chapters: Vec<ChapterCompleteness>,
}
