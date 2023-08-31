mod cli;
mod compiler;
mod introspection;
mod sqlgen;

use std::error::Error;

use compiler::Compiler;
use introspection::introspect;
pub use postgres::{Client, NoTls};

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::args();

    let mut c = Client::configure()
        .host("localhost")
        .port(5432)
        .user("postgres")
        .password("postgres")
        .dbname(&args.database)
        .connect(NoTls)?;

    let schema = introspect(&mut c, &args.schema)?;

    let tree = Compiler::new(schema);
    tree.compile_to_selects(args.table);
    Ok(())
}
