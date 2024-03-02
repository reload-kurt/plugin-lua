pub(crate) mod utils;

use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::Sender;
use std::{fs, path::Path};
  
use mlua::{ExternalError, Lua, Result, Value, Variadic};
 
pub(crate) struct Plugin {
    id: String, 
    lua: Lua,
}

impl Plugin {
    /// Create a new plugin from the path.
    /// *Note* Always assumes the correct OS-flavoured path is given
    fn new(lua: Lua, path: &str) -> Option<Plugin> {
        let full_path = String::from(path); 
        let mut plugin_name: Vec<&str> = full_path.split(utils::separator()).collect();
        // remove the entry point
        plugin_name.pop();
        
        // use the folder as the name (to prevent namespace conflicts)
        let plugin_name = plugin_name.last().unwrap().to_string();

        let script = fs::read_to_string(&full_path);
        
        if let Ok(script) = script { 
            let _ = lua.load(script.to_string())
                            .set_name(plugin_name.clone())
                            .eval::<()>();
            Some(
                Plugin {
                    id: plugin_name.clone(),   
                    lua,
                } 
            )        
        } else {
            eprintln!("unable to load scripts: {:x?}", script.err());

            None
        }
    }
}

 
pub struct PluginManager<T> {
    mem_limit: usize,
    plugins: HashMap<String, Plugin>, 
    tx: Sender<T>,
    namespaces: HashMap<String, Vec<(String, fn(Sender<T>, Variadic<Value>) -> Value)>>,
}

impl<T: 'static> PluginManager<T> {
    pub fn handle(&mut self, namespace: &str, func: &str, cb: fn(Sender<T>, Variadic<Value>) -> Value) {
        if let Some(ns) = self.namespaces.get_mut(namespace) {
            ns.push((func.to_string(), cb));
        } else {
            self.namespaces.insert(namespace.to_string(), vec![(func.to_string(), cb)]);
        }
    }

    pub fn new(mem_limit: usize, tx: Sender<T>) -> Self {
        PluginManager {
            mem_limit,
            plugins: HashMap::new(), 
            tx,
            namespaces: HashMap::new(),
        }
    } 

    /// Configure a lua context for each plugin
    pub fn configure_context(&self, lua: &Lua) -> Result<()> { 
        lua.set_memory_limit(self.mem_limit)?;
        let globals = lua.globals();
        
        for (ns, cbs) in &self.namespaces {
            let namespaced = lua.create_table()?;
            
            for cb in cbs {
                // create a communication channel between plugin manager and
                // the plugin for non-trivial calls
                let cloned_tx = self.tx.clone();
                let func = cb.1;

                namespaced.set(
                    cb.0.as_str(), 
                    lua.create_function(move |_, _vals: Variadic<Value>| {
                        Ok(func(cloned_tx.clone(), _vals))
                    })?
                )?;
            }

            globals.set(ns.as_str(), namespaced)?;
        }

        Ok(())
    }

    pub fn scan_plugins(&mut self, folder: &str, entrypoint: &str) -> bool {
        if let Ok(paths) = fs::read_dir(folder) {
            for path in paths {
                if let Ok(path) = path {
                    let npath = utils::safe_path(
                                    path.path()
                                        .to_str()
                                        .expect("cannot convert path to string")
                                );
                    
                        let ep = format!("{}{}{}", 
                            npath, 
                            utils::separator(), 
                            entrypoint
                        );
            
                        let path = Path::new(&ep);
                        
                        if path.exists() {
                            let lua = Lua::new(); 

                            if let Ok(_) = self.configure_context(&lua) {
                                if let Some(plugin) = Plugin::new(lua,  &ep) {
                                    // self.plugins.push(plugin);
                                    self.plugins.insert(
                                        plugin.id.to_string(), 
                                        plugin
                                    );
                                }
                            } else {
                                eprintln!("couldn't create Lua context for plugin");
                            }
                        } 
                } 
                // @ASSUMPTION:
                //  else do nothing? If a path is invalid we likely have bigger issues
            }

            true
        } else {   
            eprintln!("unable to scan folder {} for plugins.", folder);
            
            false
        }
    }
    
    pub fn call_plugins(&mut self, func: &str) -> bool {
        for plugin in self.plugins.values() {
            let _result = plugin.lua.load(func).exec();

            if _result.is_err() {
                let err = _result.err().unwrap();
                let err_stack = err.to_string();
                eprintln!(
                    "[{}] {}", 
                    plugin.id, 
                    err_stack
                );
                return false;
            }
        }

        true
    } 
}
