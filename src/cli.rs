use clap::Parser;
use clap_stdin::MaybeStdin;

/// A tool for exporting all data that relates to a row in a table.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// The name of the schema to connect to.
    #[arg(short, long, default_value = "public")]
    pub schema: String,

    /// The name of the database to connect to.
    #[arg(short, long, default_value = "postgres")]
    pub database: String,

    /// The name of the user to connect as.
    #[arg(short, long, default_value = "postgres")]
    pub user: String,

    /// The user's password. Prefer passing via stdin.
    #[arg(short, long, default_value = "postgres")]
    pub password: MaybeStdin<String>,

    /// The name of the table to export.
    #[arg(short, long)]
    pub table: String,

    /// The ID of the row to export.
    #[arg(long)]
    pub id: String,

    /// The name of the column that contains the ID.
    #[arg(short, long, default_value = "id")]
    pub id_column: String,
}

pub fn args() -> Args {
    Args::parse()
}
