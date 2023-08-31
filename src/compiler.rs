use std::collections::HashMap;

use la_arena::{Arena, Idx};

use crate::{introspection::RelationsInfo, introspection::Schema, sqlgen::{Select, Join, On}};

#[derive(Debug, Clone)]
pub struct TableNode {
    pub name: String,
    pub relations: Vec<TableId>,
}
type TableId = Idx<TableNode>;

#[derive(Debug, Clone)]
pub struct Compiler {
    schema: Schema,
}

impl Compiler {
    pub(crate) fn new(schema: Schema) -> Self {
        Self { schema }
    }

    pub fn build(&self) -> Arena<TableNode> {
        let mut arena = Arena::new();
        let mut table_ids = Vec::new();

        let mut table_map: HashMap<String, TableId> = std::collections::HashMap::new();

        for table in &self.schema.tables {
            let table_name = table.name.clone();
            let table_id = arena.alloc(TableNode {
                name: table_name.clone(),
                relations: Vec::new(),
            });
            table_ids.push(table_id);
            table_map.insert(table_name, table_id);
        }

        for relation in &self.schema.relations {
            let table_id = table_map.get(&relation.table_name).unwrap();
            let foreign_table_id = table_map.get(&relation.foreign_table_name).unwrap();
            let table_node = &mut arena[*table_id];

            if !table_node.relations.contains(foreign_table_id) {
                table_node.relations.push(*foreign_table_id);
            }
        }
        arena
    }

    pub(crate) fn compile_to_selects(&self, table_name: String) -> Vec<Select> {
        let arena = self.build();
        println!("{:?}", arena);
        let root = arena.iter().find(|(_idx, node)| node.name == table_name);
        if let Some(table_node) = root {
            self.traversal(&arena, &mut vec![table_node.0], &mut Vec::new())
        } else {
            Vec::new()
        }
    }

    fn traversal(
        &self,
        arena: &Arena<TableNode>,
        visited: &mut Vec<TableId>,
        selects: &mut Vec<Select>,
    ) -> Vec<Select> {
        let table_node = &arena[visited[visited.len() - 1]];
        let related_nodes = table_node.relations.iter().copied().collect::<Vec<_>>();

        let relations = self
            .schema
            .relations
            .iter()
            .filter(|relation| relation.foreign_table_name == table_node.name)
            .collect::<Vec<_>>();

        for related_node in related_nodes {
            if !visited.contains(&related_node) {
                visited.push(related_node);
                self.traversal(arena, visited, selects);
            }
        }
        println!("{:?}", visited);

        let select = Select {
            columns: self
                .schema
                .tables
                .iter()
                .find(|table| table.name == table_node.name)
                .unwrap()
                .columns
                .clone(),
            from: table_node.name.clone(),
            joins: visited.iter().rev().skip(1).map(|id| {
                Join {
                    table: arena[*id].name.clone(),
                    on: On {
                        left: relations[0].column_name.clone(),
                        right: relations[0].foreign_column_name.clone(),
                    }
                }
            }).collect::<Vec<_>>(),
        };

        visited.pop();
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::introspection::{introspect, Table};

    use super::*;

    #[test]
    fn test_traverse() {
        let tables: Vec<Table> = vec![
            Table {
                name: "users".to_string(),
                columns: vec!["id".to_string()],
            },
            Table {
                name: "posts".to_string(),
                columns: vec!["id".to_string(), "user_id".to_string()],
            },
            Table {
                name: "comments".to_string(),
                columns: vec!["id".to_string(), "post_id".to_string()],
            },
        ];
        let relations = vec![
            RelationsInfo {
                table_schema: "public".to_string(),
                foreign_table_schema: "public".to_string(),
                table_name: "users".to_string(),
                foreign_table_name: "posts".to_string(),
                column_name: "id".to_string(),
                foreign_column_name: "user_id".to_string(),
            },
            RelationsInfo {
                table_schema: "public".to_string(),
                foreign_table_schema: "public".to_string(),
                table_name: "posts".to_string(),
                foreign_table_name: "comments".to_string(),
                column_name: "id".to_string(),
                foreign_column_name: "post_id".to_string(),
            },
        ];
        let schema = Schema { relations, tables };
        let introspector = Compiler::new(schema);
        introspector.compile_to_selects("users".to_string());
    }
}
