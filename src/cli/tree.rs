
use std::collections::HashMap;
use comms::MainThreadComms;
use event::Event;

// Either a node has children, or it has a command
pub enum Node
{
    WithChildren(HashMap<String, Node>),
    DispatchEvent(fn(Vec<String>) -> Result<Event, String>),
}

impl Node {
    /// Build a new node with children
    pub fn new_with_children() -> Node {
        Node::WithChildren(HashMap::new())
    }

    /// Build a new node with a function to process
    /// any remaining tokens and return an event
    pub fn new_dispatch_event(f: fn(Vec<String>) -> Result<Event, String>) -> Node
    {
        Node::DispatchEvent(f)
    }

    /// Add a child node and return a reference to it
    pub fn add_child(&mut self, name: &str, child: Node) -> &mut Node {
        let child_map = match self {
            Node::WithChildren(map) => map,
            _ => panic!("Can't add children to a node that doesn't have a child map"),
        };
        child_map.insert(name.to_string(), child);
        child_map.get_mut(name).unwrap()
    }
}

pub struct Tree {
    root: Node,
}

impl Tree {
    pub fn new(root: Node) -> Tree {
        Tree {
            root
        }
    }

    pub fn get_current_node(&self, line: &str) -> &Node {
        let mut tokens = line.trim().split_whitespace()
            .map(|item| { item.to_string() });

        let mut current_node = &self.root;
        loop {
            match current_node {
                Node::WithChildren(child_map) => {
                    match tokens.next() {
                        Some(token) => {
                            match child_map.get(&token) {
                                Some(child) => current_node = &child,
                                None => break, // Can't match the trailing text to a node,
                                               // return the last node that we could match
                            }
                        },
                        None => break,
                    }
                },
                Node::DispatchEvent(_) => {
                    // The final node
                    break;
                },
            }
        }
        current_node

    }

    pub fn get_completion_options(&self, line: &str) -> Result<Vec<String>, ()> {
        let current_node = self.get_current_node(line);

        let child_map = match current_node {
            Node::WithChildren(child_map) => child_map,
            _ => return Err(()),
        };

        let mut completion_options = Vec::new();
        for key in child_map.keys() {
            completion_options.push(key.clone());
        }
        Ok(completion_options)
    }

    pub fn execute_command(&self, line: String, comms: &mut MainThreadComms) {
        let mut tokens = line.trim().split_whitespace()
            .map(|item| { item.to_string() });

        // Identify the right node to generate the event
        let mut current_node = &self.root;
        loop {
            match current_node {
                Node::WithChildren(map) => {
                    if let Some(token) = tokens.next() {
                        match map.get(&token) {
                            Some(child) => current_node = &child,
                            None => return, // Can't proceed, invalid command
                        }
                    }
                },
                Node::DispatchEvent(f) => {
                    // The final node. Get the event
                    let event = match f(tokens.collect()) {
                        Err(usage_msg) => {
                            println!("{}", usage_msg);
                            return
                        }
                        Ok(event) => event,
                    };
                    comms.tx.send(event).expect("Could not send event to audio thread");
                    let result = comms.rx.recv();
                    match result {
                        Ok(_) => println!("OK"),
                        Err(e) => println!("Error: {}", e),
                    }
                    // And finally
                    break
                },
            }
        }
    }
}

