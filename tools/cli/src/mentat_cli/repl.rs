// Copyright 2017 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use std::collections::HashMap;

use mentat::query::QueryResults;
use mentat_core::TypedValue;

use command_parser::{
    Command, 
    HELP_COMMAND, 
    OPEN_COMMAND
};
use input::InputReader;
use input::InputResult::{
    MetaCommand, 
    Empty, 
    More, 
    Eof
};
use store::{ 
    Store,
    db_output_name
};

lazy_static! {
    static ref COMMAND_HELP: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert(HELP_COMMAND, "Show help for commands.");
        map.insert(OPEN_COMMAND, "Open a database at path.");
        map
    };
}

/// Executes input and maintains state of persistent items.
pub struct Repl {
    store: Store,
}

impl Repl {
    /// Constructs a new `Repl`.
    pub fn new(db_name: Option<String>) -> Result<Repl, String> {
        let store = try!(Store::new(db_name.clone()).map_err(|e| e.to_string()));
        Ok(Repl{
            store: store,
        })
    }

    /// Runs the REPL interactively.
    pub fn run(&mut self) {
        let mut input = InputReader::new();

        loop {
            let res = input.read_input();

            match res {
                Ok(MetaCommand(cmd)) => {
                    debug!("read command: {:?}", cmd);
                    self.handle_command(cmd);
                },
                Ok(Empty) |
                Ok(More) => (),
                Ok(Eof) => {
                    if input.is_tty() {
                        println!("");
                    }
                    break;
                },
                Err(e) => println!("{}", e.to_string()),
            }
        }
    }

    /// Runs a single command input.
    fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::Help(args) => self.help_command(args),
            Command::Open(db) => {
                match self.store.open(Some(db.clone())) {
                    Ok(_) => println!("Database {:?} opened", db_output_name(&db)),
                    Err(e) => println!("{}", e.to_string())
                };
            },
            Command::Close => {
                let old_db_name = self.store.db_name.clone();
                match self.store.close() {
                    Ok(_) => println!("Database {:?} closed", db_output_name(&old_db_name)),
                    Err(e) => println!("{}", e.to_string())
                };
            },
            Command::Query(query) => self.query_command(query),
            Command::Transact(transaction) => self.transact_command(transaction),
            _ => unimplemented!(),
        }
    }

    fn help_command(&self, args: Vec<String>) {
        if args.is_empty() {
            for (cmd, msg) in COMMAND_HELP.iter() {
                println!(".{} - {}", cmd, msg);
            }
        } else {
            for mut arg in args {
                if arg.chars().nth(0).unwrap() == '.' { 
                    arg.remove(0);
                }
                let msg = COMMAND_HELP.get(arg.as_str());
                if msg.is_some() {
                    println!(".{} - {}", arg, msg.unwrap());
                } else {
                    println!("Unrecognised command {}", arg);
                }
            }
        }
    }

    fn query_command(&self, query: String) {
        let results = match self.store.query(query){
            Result::Ok(vals) => {
                vals
            },
            Result::Err(err) => return println!("{:?}.", err),
        };

        if results.is_empty() {
            println!("No results found.")
        }
        
        let mut output:String = String::new();
        match results {
            QueryResults::Scalar(Some(val)) => { 
                output.push_str(&self.typed_value_as_string(val) ); 
            },
            QueryResults::Tuple(Some(vals)) => { 
                for val in vals {
                    output.push_str(&format!("{}\t", self.typed_value_as_string(val)));
                }
            },
            QueryResults::Coll(vv) => { 
                for val in vv {
                    output.push_str(&format!("{}\n", self.typed_value_as_string(val)));
                }
            },
            QueryResults::Rel(vvv) => { 
                for vv in vvv {
                    for v in vv {
                        output.push_str(&format!("{}\t", self.typed_value_as_string(v)));
                    }
                    output.push_str("\n");
                }
            },
            _ => output.push_str(&format!("No results found."))
        }
        println!("\n{}", output);
    }

    fn transact_command(&mut self, transaction: String) {
        match self.store.transact(transaction) {
            Result::Ok(report) => println!("{:?}", report),
            Result::Err(err) => println!("{:?}.", err),
        }
    }

    fn typed_value_as_string(&self, value: TypedValue) -> String {
        match value {
            TypedValue::Boolean(b) => if b { "true".to_string() } else { "false".to_string() },
            TypedValue::Double(d) => format!("{}", d),
            TypedValue::Instant(i) => format!("{}", i),
            TypedValue::Keyword(k) => format!("{}", k),
            TypedValue::Long(l) => format!("{}", l),
            TypedValue::Ref(r) => format!("{}", r),
            TypedValue::String(s) => format!("{:?}", s.to_string()),
            TypedValue::Uuid(u) => format!("{}", u),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
