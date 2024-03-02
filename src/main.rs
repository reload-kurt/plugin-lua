

/// Our basic message structure to communicate between the Lua script
/// and the app
enum Message {
    Exit(bool),
}
 
use std::sync::mpsc::channel;

use mlua::Value;
use plugin_lua::*;

// All internal functions have to return a Lua Value [mlua::Value]
// for void return Value::Nil
fn add_internal_functions(pm: &mut PluginManager<Message>) {

    // Add an add function
    pm.handle("math", "add", |_, v| {
        let a = v.get(0).expect("missing a");
        let b = v.get(1).expect("missing b"); 

        Value::Number(a.as_f64().unwrap() + b.as_f64().unwrap())
    });

    // Add the exit function
    pm.handle("sys", "exit", |t, _| {
        t.send(Message::Exit(true)).unwrap();

        Value::Nil
    });

    // Add a print function
    pm.handle("sys", "print", |_, a| {
        for v in a {
            match v {
                Value::Boolean(b) => print!("{}", b),
                Value::Integer(i) => print!("{}", i),
                Value::Number(n) => print!("{}", n),
                Value::String(s) => print!("{}", s.to_str().unwrap()),
                Value::Nil => print!("[nil]"),
                _ => {},
            };
        }

        print!("\n");

        Value::Nil
    });
}

fn main() {
    let (tx, rx) = channel::<Message>(); 

    // crate a plugin manager with a 2mb limit for each plugin
    let mut pm = PluginManager::new(2_048_000, tx);   
    // we need to populate all our internals first
    add_internal_functions(&mut pm);
    
    // scan the folder for plugins
    pm.scan_plugins("./plugins", "main.lua");

    // Let's call all the plugins' init function
    pm.call_plugins("init()");
    
    // enter the main loop and loop until we're asked not to either
    // by an internal message or a plugin returning false on update
    'mainloop: loop {
        if !pm.call_plugins("update()") {
            break;
        } 

        loop {
            match rx.try_recv() {
                Ok(msg) => {
                    match msg {
                        Message::Exit(b) => if b { break 'mainloop },
                    }
                },
                _ => break, // break the inner loop and continue frames
            }
        }
    }
    
    // call any cleanup functions inside the plugins
    pm.call_plugins("destroy()");
}
