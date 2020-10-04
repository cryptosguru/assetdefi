use lru::LruCache;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use wasmi::*;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;

/// An abstraction of transaction execution state.
///
/// It acts as the facade of ledger state and keeps track of all temporary state updates,
/// until the `commit()` method is called.
///
/// Typically, a track is shared by all the processes created within a transaction.
///
pub struct Track<'l, L: Ledger> {
    ledger: &'l mut L,
    current_epoch: u64,
    tx_hash: H256,
    id_alloc: IdAllocator,
    logs: Vec<(Level, String)>,
    packages: HashMap<Address, Package>,
    components: HashMap<Address, Component>,
    resource_defs: HashMap<Address, ResourceDef>,
    lazy_maps: HashMap<Mid, LazyMap>,
    vaults: HashMap<Vid, Vault>,
    updated_packages: HashSet<Address>,
    updated_components: HashSet<Address>,
    updated_lazy_maps: HashSet<Mid>,
    updated_resource_defs: HashSet<Address>,
    updated_vaults: HashSet<Vid>,
    new_entities: Vec<Address>,
    code_cache: LruCache<Address, Module>, // TODO: move to ledger level
}

impl<'l, L: Ledger> Track<'l, L> {
    pub fn new(ledger: &'l mut L, current_epoch: u64, tx_hash: H256) -> Self {
        Self {
            ledger,
            current_epoch,
            tx_hash,
            id_alloc: IdAllocator::new(),
            logs: Vec::new(),
            packages: HashMap::new(),
            components: HashMap::new(),
            resource_defs: HashMap::new(),
            lazy_maps: HashMap::new(),
            vaults: HashMap::new(),
            updated_packages: HashSet::new(),
            updated_components: HashSet::new(),
            updated_lazy_maps: HashSet::new(),
            updated_resource_defs: HashSet::new(),
            updated_vaults: HashSet::new(),
            new_entities: Vec::new(),
            code_cache: LruCache::new(1024),
        }
    }

    /// Start a process.
    pub fn start_process<'r>(&'r mut self, verbose: bool) -> Process<'r, 'l, L> {
        Process::new(0, verbose, self)
    }

    /// Returns the transaction hash.
    pub fn tx_hash(&self) -> H256 {
        self.tx_hash
    }

    /// Returns the current epoch.
    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Returns the logs collected so far.
    pub fn logs(&self) -> &Vec<(Level, String)> {
        &self.logs
    }

    /// Returns new entities created so far.
    pub fn new_entities(&self) -> &[Address] {
        &self.new_entities
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: Level, message: String) {
        self.logs.push((level, message));
    }

    /// Loads a module.
    pub fn load_module(&mut self, address: Address) -> Option<(ModuleRef, MemoryRef)> {
        match self.get_package(address).map(Clone::clone) {
            Some(p) => {
                if let Some(m) = self.code_cache.get(&address) {
                    Some(instantiate_module(m).unwrap())
                } else {
                    let module = parse_module(p.code()).unwrap();
                    let inst = instantiate_module(&module).unwrap();
                    self.code_cache.put(address, module);
                    Some(inst)
                }
            }
            None => None,
        }
    }

    /// Returns an immutable reference to a package, if exists.
    pub fn get_package(&mut self, address: Address) -> Option<&Package> {
        if self.packages.contains_key(&address) {
            return self.packages.get(&address);
        }

        if let Some(package) = self.ledger.get_package(address) {
            self.packages.insert(address, package);
            self.packages.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a package, if exists.
    #[allow(dead_code)]
    pub fn get_package_mut(&mut self, address: Address) -> Option<&mut Package> {
        self.updated_packages.insert(address);

        if self.packages.contains_key(&address) {
            return self.packages.get_mut(&address);
        }

        if let Some(package) = self.ledger.get_package(address) {
            self.packages.insert(address, package);
            self.packages.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new package.
    pub fn put_package(&mut self, address: Address, package: Package) {
        self.updated_packages.insert(address);

        self.packages.insert(address, package);
    }

    /// Returns an immutable reference to a component, if exists.
    pub fn get_component(&mut self, address: Address) -> Option<&Component> {
        if self.components.contains_key(&address) {
            return self.components.get(&address);
        }

        if let Some(component) = self.ledger.get_component(address) {
            self.components.insert(address, component);
            self.components.get(&address)
        } else {
            None
        }
    }
    /// Returns a mutable reference to a component, if exists.
    pub fn get_component_mut(&mut self, address: Address) -> Option<&mut Component> {
        self.updated_components.insert(address);

        if self.components.contains_key(&address) {
            return self.components.get_mut(&address);
        }

        if let Some(component) = self.ledger.get_component(address) {
            self.components.insert(address, component);
            self.components.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new component.
    pub fn put_component(&mut self, address: Address, component: Component) {
        self.updated_components.insert(address);

        self.components.insert(address, component);
    }

    /// Returns an immutable reference to a lazy map, if exists.
    pub fn get_lazy_map(&mut self, mid: Mid) -> Option<&LazyMap> {
        if self.lazy_maps.contains_key(&mid) {
            return self.lazy_maps.get(&mid);
        }

        if let Some(lazy_map) = self.ledger.get_lazy_map(mid) {
            self.lazy_maps.insert(mid, lazy_map);
            self.lazy_maps.get(&mid)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a lazy map, if exists.
    pub fn get_lazy_map_mut(&mut self, mid: Mid) -> Option<&mut LazyMap> {
        self.updated_lazy_maps.insert(mid);

        if self.lazy_maps.contains_key(&mid) {
            return self.lazy_maps.get_mut(&mid);
        }

        if let Some(lazy_map) = self.ledger.get_lazy_map(mid) {
            self.lazy_maps.insert(mid, lazy_map);
            self.lazy_maps.get_mut(&mid)
        } else {
            None
        }
    }

    /// Inserts a new lazy map.
    pub fn put_lazy_map(&mut self, mid: Mid, lazy_map: LazyMap) {
        self.updated_lazy_maps.insert(mid);

        self.lazy_maps.insert(mid, lazy_map);
    }

    /// Returns an immutable reference to a resource definition, if exists.
    pub fn get_resource_def(&mut self, address: Address) -> Option<&ResourceDef> {
        if self.resource_defs.contains_key(&address) {
            return self.resource_defs.get(&address);
        }

        if let Some(resource_def) = self.ledger.get_resource_def(address) {
            self.resource_defs.insert(address, resource_def);
            self.resource_defs.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a resource definition, if exists.
    #[allow(dead_code)]
    pub fn get_resource_def_mut(&mut self, address: Address) -> Option<&mut ResourceDef> {
        self.updated_resource_defs.insert(address);

        if self.resource_defs.contains_key(&address) {
            return self.resource_defs.get_mut(&address);
        }

        if let Some(resource_def) = self.ledger.get_resource_def(address) {
            self.resource_defs.insert(address, resource_def);
            self.resource_defs.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new resource definition.
    pub fn put_resource_def(&mut self, address: Address, resource_def: ResourceDef) {
        self.updated_resource_defs.insert(address);

        self.resource_defs.insert(address, resource_def);
    }

    /// Returns an immutable reference to a vault, if exists.
    #[allow(dead_code)]
    pub fn get_vault(&mut self, vid: Vid) -> Option<&Vault> {
        if self.vaults.contains_key(&vid) {
            return self.vaults.get(&vid);
        }

        if let Some(vault) = self.ledger.get_vault(vid) {
            self.vaults.insert(vid, vault);
            self.vaults.get(&vid)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a vault, if exists.
    pub fn get_vault_mut(&mut self, vid: Vid) -> Option<&mut Vault> {
        self.updated_vaults.insert(vid);

        if self.vaults.contains_key(&vid) {
            return self.vaults.get_mut(&vid);
        }

        if let Some(vault) = self.ledger.get_vault(vid) {
            self.vaults.insert(vid, vault);
            self.vaults.get_mut(&vid)
        } else {
            None
        }
    }

    /// Inserts a new vault.
    pub fn put_vault(&mut self, vid: Vid, vault: Vault) {
        self.updated_vaults.insert(vid);

        self.vaults.insert(vid, vault);
    }

    /// Creates a new package address.
    pub fn new_package_address(&mut self) -> Address {
        let address = self.id_alloc.new_package_address(self.tx_hash());
        self.new_entities.push(address);
        address
    }

    /// Creates a new component address.
    pub fn new_component_address(&mut self) -> Address {
        let address = self.id_alloc.new_component_address(self.tx_hash());
        self.new_entities.push(address);
        address
    }

    /// Creates a new resource definition address.
    pub fn new_resource_def_address(&mut self) -> Address {
        let address = self.id_alloc.new_resource_def_address(self.tx_hash());
        self.new_entities.push(address);
        address
    }

    /// Creates a new bucket ID.
    pub fn new_bid(&mut self) -> Bid {
        self.id_alloc.new_bid()
    }

    /// Creates a new vault ID.
    pub fn new_vid(&mut self) -> Vid {
        self.id_alloc.new_vid(self.tx_hash())
    }

    /// Creates a new reference id.
    pub fn new_rid(&mut self) -> Rid {
        self.id_alloc.new_rid()
    }

    /// Creates a new map id.
    pub fn new_mid(&mut self) -> Mid {
        self.id_alloc.new_mid(self.tx_hash())
    }

    /// Commits changes to the underlying ledger.
    pub fn commit(&mut self) {
        for address in self.updated_packages.clone() {
            self.ledger
                .put_package(address, self.packages.get(&address).unwrap().clone());
        }

        for address in self.updated_components.clone() {
            self.ledger
                .put_component(address, self.components.get(&address).unwrap().clone());
        }

        for address in self.updated_resource_defs.clone() {
            self.ledger
                .put_resource_def(address, self.resource_defs.get(&address).unwrap().clone());
        }

        for mid in self.updated_lazy_maps.clone() {
            self.ledger
                .put_lazy_map(mid, self.lazy_maps.get(&mid).unwrap().clone());
        }

        for vault in self.updated_vaults.clone() {
            self.ledger
                .put_vault(vault, self.vaults.get(&vault).unwrap().clone());
        }
    }
}
