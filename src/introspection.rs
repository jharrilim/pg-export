use postgres::{Client, NoTls, Row};

fn table_names_query(schema: &str) -> String {
    format!(
        r#"
          SELECT table_name
          FROM information_schema.tables
          WHERE table_schema = '{}'
          AND table_type = 'BASE TABLE'
          ORDER BY table_name;
        "#,
        schema
    )
}

fn columns_query(table_name: String) -> String {
    format!(
        r#"
          SELECT column_name
          FROM information_schema.columns
          WHERE table_name = '{}'
          ORDER BY ordinal_position;
        "#,
        table_name
    )
}

const RELATIONS_QUERY: &'static str = r#"
SELECT
  tc.table_schema,
  tc.table_name,
  kcu.column_name,
  ccu.table_schema AS foreign_table_schema,
  ccu.table_name AS foreign_table_name,
  ccu.column_name AS foreign_column_name
FROM
  information_schema.table_constraints AS tc
  JOIN information_schema.key_column_usage AS kcu
    ON tc.constraint_name = kcu.constraint_name
    AND tc.table_schema = kcu.table_schema
  JOIN information_schema.constraint_column_usage AS ccu
    ON ccu.constraint_name = tc.constraint_name
    AND ccu.table_schema = tc.table_schema
WHERE tc.constraint_type = 'FOREIGN KEY';
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationsInfo {
    pub table_schema: String,
    pub table_name: String,
    pub column_name: String,
    pub foreign_table_schema: String,
    pub foreign_table_name: String,
    pub foreign_column_name: String,
}

impl From<Row> for RelationsInfo {
    fn from(row: Row) -> Self {
        Self {
            table_schema: row.get(0),
            table_name: row.get(1),
            column_name: row.get(2),
            foreign_table_schema: row.get(3),
            foreign_table_name: row.get(4),
            foreign_column_name: row.get(5),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub name: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub tables: Vec<Table>,
    pub relations: Vec<RelationsInfo>,
}

pub fn introspect(
    client: &mut Client,
    schema: &str,
) -> Result<Schema, Box<dyn std::error::Error>> {
    let rows = client.query(RELATIONS_QUERY, &[])?;
    let relations: Vec<RelationsInfo> = rows
        .into_iter()
        .map(|row| RelationsInfo::from(row))
        .collect();

    let table_names = client.query(table_names_query(schema).as_str(), &[])?;
    let table_names = table_names
        .into_iter()
        .map(|row| row.get(0))
        .collect::<Vec<String>>();

    let mut tables = Vec::new();
    for table_name in table_names {
        let columns = client.query(columns_query(table_name.clone()).as_str(), &[])?;
        let columns = columns
            .into_iter()
            .map(|row| row.get(0))
            .collect::<Vec<String>>();
        tables.push(Table {
            name: table_name,
            columns,
        });
    }
    Ok(Schema { tables, relations })
}
