
use event::Event;
use std::collections::HashMap;
use std::str::SplitWhitespace;
use std::sync::mpsc;

// Either a node has children, or it has a command
pub enum Node
{
    WithChildren(HashMap<String, Node>),
    DispatchEvent(fn(&mut SplitWhitespace) -> Result<Event, String>),
}

impl Node {
    /// Build a new node with children
    pub fn new_with_children() -> Node {
        Node::WithChildren(HashMap::new())
    }

    /// Build a new node with a function to process
    /// any remaining tokens and return an event
    pub fn new_dispatch_event(f: fn(&mut SplitWhitespace) -> Result<Event, String>) -> Node
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
        let mut tokens = line.trim().split_whitespace();

        let mut current_node = &self.root;
        loop {
            match current_node {
                Node::WithChildren(child_map) => {
                    match tokens.next() {
                        Some(token) => {
                            match child_map.get(token) {
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

    pub fn execute_command(&self,
                           line: String,
                           tx: &mpsc::SyncSender<Event>,
                           rx: &mpsc::Receiver<Result<(), &'static str>>,
    ) {
        let mut tokens = line.trim().split_whitespace();

        // Identify the right node to generate the event
        let mut current_node = &self.root;
        loop {
            match current_node {
                Node::WithChildren(child_map) => {
                    let token = match tokens.next() {
                        None => return, // Can't proceed, would need more tokens
                        Some(token) => token,
                    };
                    match child_map.get(token) {
                        None => return, // Can't proceed, invalid command
                        Some(child) => {
                            // assign this child as the current node and continue looping
                            current_node = &child;
                        }
                    }
                },
                Node::DispatchEvent(f) => {
                    // The final node. Get the event
                    let event = match f(&mut tokens) {
                        Err(usage_msg) => {
                            println!("{}", usage_msg);
                            return
                        }
                        Ok(event) => event,
                    };
                    // Send the event to the audio thread, then handle the response
                    tx.send(event).expect("Could not send event to audio thread");
                    match rx.recv() {
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

