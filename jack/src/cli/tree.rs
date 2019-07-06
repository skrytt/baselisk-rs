
use rustyline::completion::Pair;
use baselisk_core::shared::SharedState;
use std::collections::HashMap;
use std::str::SplitWhitespace;
use std::sync::Arc;

// Either a node has children, or it has a command
pub enum Node
{
    WithChildren(HashMap<String, Node>),
    DispatchEvent(fn(&mut SplitWhitespace, &Arc<SharedState>) -> Result<(), String>,
                  Option<String>),
}

impl Node {
    /// Build a new node with children
    pub fn new_with_children() -> Self {
        Node::WithChildren(HashMap::new())
    }

    /// Build a new node with a function to process
    /// any remaining tokens and return an event
    pub fn new_dispatch_event(f: fn(&mut SplitWhitespace,
                                    &Arc<SharedState>) -> Result<(), String>,
                              argument_hint: Option<String>) -> Self
    {
        Node::DispatchEvent(f, argument_hint)
    }

    /// Add a child node and return a reference to it
    pub fn add_child(&mut self, name: &str, child: Self) -> &mut Self {
        let child_map = match self {
            Node::WithChildren(map) => map,
            _ => panic!("Can't add children to a node that doesn't have a child map"),
        };
        child_map.insert(name.to_string(), child);
        child_map.get_mut(name).unwrap()
    }

    pub fn get_argument_hint(&self) -> Option<String> {
        match self {
            Node::DispatchEvent(_f, hint) => hint.clone(),
            _ => None,
        }
    }
}

pub struct Tree {
    root: Node,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Self {
            root
        }
    }

    pub fn get_current_node(&self, line: &str) -> &Node {
        let mut tokens = line.trim().split_whitespace();

        let mut current_node = &self.root;

        while let Node::WithChildren(child_map) = current_node {
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
        }
        current_node

    }

    pub fn get_completion_options(&self, line: &str) -> Result<Vec<Pair>, ()> {
        let current_node = self.get_current_node(line);

        let child_map = match current_node {
            Node::WithChildren(child_map) => child_map,
            _ => return Err(()),
        };

        let mut completion_options = Vec::new();
        for (key, child_node) in child_map.iter() {
            // Push a space to the end of the replacement, so that autocompleting this token
            // in full allows the user to then begin autocompleting the next token
            let mut replacement = key.clone();
            replacement.push(' ');

            // Display should start with the replacement string and then hint at
            // what to type next
            let mut display = replacement.clone();
            if let Some(hint) = child_node.get_argument_hint() {
                display.push_str(&hint);
            }
            let pair = Pair {
                display,
                replacement,
            };

            completion_options.push(pair);
        }
        Ok(completion_options)
    }

    pub fn execute_command(&self,
                           line: &str,
                           shared_state: &Arc<SharedState>,
    ) {
        let line = line.trim();

        // If nothing was typed, don't bother printing an error message
        if line.is_empty() {
            return
        }

        let mut tokens = line.trim().split_whitespace();

        // Identify the right node to generate the event
        let mut current_node = &self.root;
        loop {
            match current_node {
                Node::WithChildren(child_map) => {
                    let token = match tokens.next() {
                        None => {
                            println!("Error: Expected more tokens in command!");
                            return
                        }
                        Some(token) => token,
                    };

                    match child_map.get(token) {
                        None => {
                            println!("Error: unrecognised token '{}'!", token);
                            return
                        }
                        Some(child) => {
                            // assign this child as the current node and continue looping
                            current_node = &child;
                        }
                    }
                },
                Node::DispatchEvent(f, _hint) => {
                    // The final node. Get the event
                    match f(&mut tokens, shared_state) {
                        Err(usage_msg) => {
                            println!("Error: {}", usage_msg);
                            return
                        }
                        Ok(_) => (),
                    };

                    // And finally
                    break
                },
            }
        }
    }
}

